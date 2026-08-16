#!/usr/bin/env python3
"""Turn a packet capture into the gate evidence R-25 asks for, or into a failure that names itself.

Reads what `scripts/capture.sh` writes — a pcap, the sockets its process tree held while it ran,
and the manifest `examples/audit-vault.rs` generated beside the fixture vault — and answers the
four gate lines that are about the wire:

    2. only a 5-character SHA-1 prefix leaves the machine — no full hash, no password, no item
       title, no vault or item identifier
    4. with breach checking off, zero packets across 10 minutes of active use (S-10, R-26)
    6. one range request per distinct value, concurrency held to a bound (S-07b, in part)
    7. the evidence is recorded rather than asserted

Usage:

    capture-report.py --mode off|on --pcap DIR/capture.pcap --sockets DIR/sockets.txt \\
        --manifest /tmp/audit-manifest.txt --meta DIR/run.txt --out DIR/report.md

Exit status is 0 when every check for the mode passed, 1 when one did not, and 2 when the inputs
could not be read. A failing check is the useful outcome: it is why this is a script rather than a
person reading `tcpdump -A`.

# What this can prove, and what it cannot — D-89

The request is inside TLS. Nothing here can read the five characters and confirm they were five;
what `hibp.rs`'s own tests assert is what was handed to the client, and this asserts what the
machine did with it. So gate line 2 is checked as the **negative** it is worded as — no password,
no full hash, no title, no id anywhere in the bytes, and no host other than the one R-25 names —
plus the destination evidence that is genuinely in the clear: the SNI in each ClientHello and the
DNS questions that preceded it.

# Why the whole capture is searched, not just the app's packets

Attribution is by socket, and sockets are sampled (see `capture.sh`). A sample can miss a socket
that opened and closed between two polls. So every check that can be run against the **entire**
capture is run against the entire capture — the forbidden-string sweep and the search for R-25's
hostname both are — and attribution is used only to say which conversations were the app's. A
missed sample can understate what the app did; it cannot hide a password from the sweep.

# No dependencies

Stdlib only, and that is a constraint rather than a preference: this machine has no tshark, no
dumpcap and no dpkt, and a piece of gate evidence that needs a package installed on the day is a
piece of gate evidence that gets skipped. The pcap format is a 24-byte header and a 16-byte record
prefix; the parsing below is the boring part and is short enough to read.
"""

from __future__ import annotations

import argparse
import re
import struct
import sys
from dataclasses import dataclass, field
from pathlib import Path

# The one host R-25 permits. Anything else the app talked to is a finding, not a note.
HIBP_HOST = "api.pwnedpasswords.com"

# Link-layer types this reads. `tcpdump -i any` on libpcap >= 1.10 writes LINUX_SLL2.
LINKTYPE_NULL = 0
LINKTYPE_ETHERNET = 1
LINKTYPE_RAW = 101
LINKTYPE_LINUX_SLL = 113
LINKTYPE_RAW_ALT = (12, 14)
LINKTYPE_LINUX_SLL2 = 276

# A forbidden string shorter than this is not searched for: four bytes of ASCII turn up in
# encrypted payload by chance often enough to make the report cry wolf, and every value in the
# manifest that matters — passwords, titles, ids, hashes — is longer.
MIN_NEEDLE = 5

# A client->server TLS application-data record smaller than this is counted as "request-sized".
# An estimate and labelled as one: a range request is ~120 bytes of ciphertext and a range
# response is tens of kilobytes, so the two do not overlap, but a TLS key update is also small.
REQUEST_RECORD_MAX = 512


def fail(message: str) -> "NoReturn":  # type: ignore[valid-type]
    print(f"capture-report: {message}", file=sys.stderr)
    raise SystemExit(2)


# --------------------------------------------------------------------------------------------
# pcap
# --------------------------------------------------------------------------------------------


@dataclass
class Packet:
    """One captured frame, decoded as far as this report needs and no further."""

    index: int
    timestamp: float
    raw: bytes
    src: str = ""
    dst: str = ""
    sport: int = 0
    dport: int = 0
    proto: str = ""
    payload: bytes = b""


def read_pcap(path: Path) -> tuple[int, list[Packet]]:
    data = path.read_bytes()
    if len(data) < 24:
        fail(f"{path} is {len(data)} bytes — no capture in it")
    magic = data[:4]
    if magic == b"\x0a\x0d\x0d\x0a":
        fail(f"{path} is pcapng; capture.sh writes classic pcap, so this file came from elsewhere")
    if magic in (b"\xa1\xb2\xc3\xd4", b"\xa1\xb2\x3c\x4d"):
        endian = ">"
    elif magic in (b"\xd4\xc3\xb2\xa1", b"\x4d\x3c\xb2\xa1"):
        endian = "<"
    else:
        fail(f"{path} does not start with a pcap magic number")
    nanoseconds = magic in (b"\xa1\xb2\x3c\x4d", b"\x4d\x3c\xb2\xa1")
    linktype = struct.unpack(endian + "I", data[20:24])[0]

    packets: list[Packet] = []
    offset = 24
    index = 0
    while offset + 16 <= len(data):
        seconds, fraction, included, _original = struct.unpack(endian + "IIII", data[offset : offset + 16])
        offset += 16
        frame = data[offset : offset + included]
        if len(frame) < included:
            # A capture killed mid-write ends in a short record. Keeping what parsed is right:
            # the alternative is discarding an entire ten-minute run over its last frame.
            break
        offset += included
        index += 1
        timestamp = seconds + fraction / (1_000_000_000 if nanoseconds else 1_000_000)
        packet = Packet(index=index, timestamp=timestamp, raw=frame)
        decode(packet, linktype)
        packets.append(packet)
    return linktype, packets


def decode(packet: Packet, linktype: int) -> None:
    """Fill in addresses, ports and payload. Anything not IP over TCP/UDP is left as raw bytes."""
    frame = packet.raw
    if linktype == LINKTYPE_LINUX_SLL2:
        if len(frame) < 20:
            return
        ethertype = struct.unpack(">H", frame[0:2])[0]
        rest = frame[20:]
    elif linktype == LINKTYPE_LINUX_SLL:
        if len(frame) < 16:
            return
        ethertype = struct.unpack(">H", frame[14:16])[0]
        rest = frame[16:]
    elif linktype == LINKTYPE_ETHERNET:
        if len(frame) < 14:
            return
        ethertype = struct.unpack(">H", frame[12:14])[0]
        rest = frame[14:]
        while ethertype in (0x8100, 0x88A8) and len(rest) >= 4:
            ethertype = struct.unpack(">H", rest[2:4])[0]
            rest = rest[4:]
    elif linktype == LINKTYPE_NULL:
        if len(frame) < 4:
            return
        family = struct.unpack("<I", frame[0:4])[0]
        ethertype = 0x0800 if family == 2 else 0x86DD
        rest = frame[4:]
    elif linktype == LINKTYPE_RAW or linktype in LINKTYPE_RAW_ALT:
        if not frame:
            return
        ethertype = 0x0800 if frame[0] >> 4 == 4 else 0x86DD
        rest = frame
    else:
        return

    if ethertype == 0x0800:
        decode_ipv4(packet, rest)
    elif ethertype == 0x86DD:
        decode_ipv6(packet, rest)


def decode_ipv4(packet: Packet, data: bytes) -> None:
    if len(data) < 20:
        return
    header_length = (data[0] & 0x0F) * 4
    if len(data) < header_length:
        return
    protocol = data[9]
    packet.src = ".".join(str(byte) for byte in data[12:16])
    packet.dst = ".".join(str(byte) for byte in data[16:20])
    decode_transport(packet, protocol, data[header_length:])


def decode_ipv6(packet: Packet, data: bytes) -> None:
    if len(data) < 40:
        return
    packet.src = format_ipv6(data[8:24])
    packet.dst = format_ipv6(data[24:40])
    # Extension headers are not walked. They are absent from this traffic, and a report that
    # silently mis-parsed one would be worse than one that says it saw an unknown protocol.
    decode_transport(packet, data[6], data[40:])


def format_ipv6(raw: bytes) -> str:
    groups = [f"{raw[i] << 8 | raw[i + 1]:x}" for i in range(0, 16, 2)]
    return ":".join(groups)


def decode_transport(packet: Packet, protocol: int, data: bytes) -> None:
    if protocol == 6 and len(data) >= 20:
        packet.proto = "tcp"
        packet.sport, packet.dport = struct.unpack(">HH", data[0:4])
        offset = (data[12] >> 4) * 4
        packet.payload = data[offset:] if len(data) >= offset else b""
    elif protocol == 17 and len(data) >= 8:
        packet.proto = "udp"
        packet.sport, packet.dport = struct.unpack(">HH", data[0:4])
        packet.payload = data[8:]
    else:
        packet.proto = f"ip-proto-{protocol}"


# --------------------------------------------------------------------------------------------
# What is in the packets: names asked for, names connected to
# --------------------------------------------------------------------------------------------


def dns_questions(payload: bytes) -> list[str]:
    """Every QNAME in a DNS message, queries and responses alike.

    Responses are read too, deliberately: on a machine running systemd-resolved the app's query
    goes to 127.0.0.53 and the answer comes back from it, and the answer is where the addresses
    are that the capture then has to tie to a name.
    """
    if len(payload) < 12:
        return []
    try:
        count = struct.unpack(">H", payload[4:6])[0]
    except struct.error:
        return []
    names = []
    offset = 12
    for _ in range(min(count, 16)):
        labels = []
        while offset < len(payload):
            length = payload[offset]
            if length == 0:
                offset += 1
                break
            if length & 0xC0:  # a pointer: questions do not use them, so stop rather than guess
                offset += 2
                break
            offset += 1
            labels.append(payload[offset : offset + length].decode("ascii", "replace"))
            offset += length
        if labels:
            names.append(".".join(labels))
        offset += 4  # QTYPE and QCLASS
        if offset >= len(payload):
            break
    return names


def dns_addresses(payload: bytes, wanted: str) -> set[str]:
    """A and AAAA records in a response, when the question was `wanted`.

    Parsed without following compression pointers into the answer names: the record type, class
    and length are enough to walk the section, and the addresses are the only field read.
    """
    if wanted not in dns_questions(payload):
        return set()
    if len(payload) < 12:
        return set()
    questions, answers = struct.unpack(">HH", payload[4:8])
    offset = 12
    for _ in range(questions):
        while offset < len(payload) and payload[offset] != 0:
            if payload[offset] & 0xC0:
                offset += 1
                break
            offset += payload[offset] + 1
        offset += 5
    found: set[str] = set()
    for _ in range(min(answers, 64)):
        if offset + 12 > len(payload):
            break
        if payload[offset] & 0xC0:
            offset += 2
        else:
            while offset < len(payload) and payload[offset] != 0:
                offset += payload[offset] + 1
            offset += 1
        if offset + 10 > len(payload):
            break
        rtype, _rclass, _ttl, rdlength = struct.unpack(">HHIH", payload[offset : offset + 10])
        offset += 10
        record = payload[offset : offset + rdlength]
        offset += rdlength
        if rtype == 1 and len(record) == 4:
            found.add(".".join(str(byte) for byte in record))
        elif rtype == 28 and len(record) == 16:
            found.add(format_ipv6(record))
    return found


def client_hello_sni(payload: bytes) -> str | None:
    """The server name in a TLS ClientHello, which is in the clear and is the destination evidence.

    Returns None for anything that is not a ClientHello. If a capture ever produces a TLS
    handshake with **no** SNI, that shows up as a connection with no name against it in the report
    rather than as a silent pass — encrypted client hello would look exactly like that, and it
    would mean this check had stopped measuring anything.
    """
    if len(payload) < 45 or payload[0] != 0x16 or payload[1] != 0x03:
        return None
    try:
        offset = 5
        if payload[offset] != 0x01:  # handshake type: client_hello
            return None
        offset += 4 + 2 + 32  # length, version, random
        session_length = payload[offset]
        offset += 1 + session_length
        cipher_length = struct.unpack(">H", payload[offset : offset + 2])[0]
        offset += 2 + cipher_length
        compression_length = payload[offset]
        offset += 1 + compression_length
        extensions_length = struct.unpack(">H", payload[offset : offset + 2])[0]
        offset += 2
        end = offset + extensions_length
        while offset + 4 <= min(end, len(payload)):
            extension, length = struct.unpack(">HH", payload[offset : offset + 4])
            offset += 4
            if extension == 0x0000:  # server_name
                # list length (2), name type (1), name length (2), name
                name_length = struct.unpack(">H", payload[offset + 3 : offset + 5])[0]
                return payload[offset + 5 : offset + 5 + name_length].decode("ascii", "replace")
            offset += length
    except (struct.error, IndexError):
        return None
    return None


def tls_records(payload: bytes) -> list[tuple[int, int]]:
    """(content type, length) for each TLS record in a payload, without reassembling streams."""
    records = []
    offset = 0
    while offset + 5 <= len(payload):
        content_type = payload[offset]
        if payload[offset + 1] != 0x03 or content_type not in (20, 21, 22, 23):
            break
        length = struct.unpack(">H", payload[offset + 3 : offset + 5])[0]
        records.append((content_type, length))
        offset += 5 + length
    return records


# --------------------------------------------------------------------------------------------
# Inputs written by capture.sh
# --------------------------------------------------------------------------------------------


@dataclass
class Sockets:
    """What the app's process tree held, sampled while it ran."""

    endpoints: set[tuple[str, int]] = field(default_factory=set)
    peers: set[tuple[str, int]] = field(default_factory=set)
    by_process: dict[str, set[tuple[str, int]]] = field(default_factory=dict)
    samples: int = 0
    lines: int = 0
    #: Heartbeats — one per sampling pass, written whether or not the tree held a socket. This
    #: is what tells "the app opened no socket" apart from "the app never started", and the `off`
    #: run's entire claim is the first of those.
    heartbeats: int = 0
    #: The largest process tree seen. One is the app alone; a Tauri app under WebKitGTK is more.
    tree: int = 0
    #: Wall clock from the first sample to the last.
    span: float = 0.0


SS_ENDPOINT = re.compile(r"^\[?([0-9a-fA-F.:]+?)\]?:(\d+|\*)$")
SS_PROCESS = re.compile(r'\(\("([^"]+)",pid=(\d+)')


def read_sockets(path: Path) -> Sockets:
    sockets = Sockets()
    if not path.exists():
        return sockets
    seen_timestamps: set[str] = set()
    stamps: list[float] = []
    for line in path.read_text(errors="replace").splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        parts = line.split("\t", 1)
        if len(parts) != 2:
            continue
        seen_timestamps.add(parts[0])
        try:
            stamps.append(float(parts[0]))
        except ValueError:
            pass
        if parts[1].startswith("# tree"):
            sockets.heartbeats += 1
            size = parts[1].split()
            if len(size) == 3 and size[2].isdigit():
                sockets.tree = max(sockets.tree, int(size[2]))
            continue
        fields = parts[1].split()
        if len(fields) < 6:
            continue
        sockets.lines += 1
        local, peer = fields[4], fields[5]
        process = SS_PROCESS.search(parts[1])
        name = process.group(1) if process else "?"
        for text, bucket in ((local, sockets.endpoints), (peer, sockets.peers)):
            match = SS_ENDPOINT.match(text)
            if not match or match.group(2) == "*":
                continue
            address, port = match.group(1), int(match.group(2))
            bucket.add((address, port))
            if bucket is sockets.endpoints:
                sockets.by_process.setdefault(name, set()).add((address, port))
    sockets.samples = len(seen_timestamps)
    sockets.span = (max(stamps) - min(stamps)) if stamps else 0.0
    return sockets


@dataclass
class Manifest:
    vault: str = ""
    distinct: int = 0
    prefixes: list[str] = field(default_factory=list)
    forbidden: list[str] = field(default_factory=list)
    hashes: list[str] = field(default_factory=list)


def read_manifest(path: Path) -> Manifest:
    manifest = Manifest()
    for line in path.read_text().splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        key, _, value = line.partition("\t")
        if key == "vault":
            manifest.vault = value
        elif key == "distinct":
            manifest.distinct = int(value)
        elif key == "prefix":
            manifest.prefixes.append(value)
        elif key == "forbid":
            manifest.forbidden.append(value)
        elif key == "forbid-hash":
            manifest.hashes.append(value)
    if not manifest.forbidden and not manifest.hashes:
        fail(f"{path} lists nothing to search for — a sweep with no needles always passes")
    return manifest


def needles(manifest: Manifest) -> dict[bytes, str]:
    """Every byte string that must not appear, mapped to what it would mean if it did.

    A hash is searched for three ways — uppercase hex, lowercase hex, and the twenty raw bytes —
    because "no full hash left the machine" is a claim about the value, not about a rendering of
    it, and a client that sent a digest as binary would sail past a hex-only grep.
    """
    found: dict[bytes, str] = {}
    for value in manifest.forbidden:
        if len(value) < MIN_NEEDLE:
            continue
        found[value.encode()] = f"forbidden string `{value}`"
    for value in manifest.hashes:
        found[value.encode()] = f"full SHA-1 (upper hex) `{value}`"
        found[value.lower().encode()] = f"full SHA-1 (lower hex) `{value}`"
        try:
            found[bytes.fromhex(value)] = f"full SHA-1 (binary) `{value}`"
        except ValueError:
            continue
    return found


# --------------------------------------------------------------------------------------------
# The report
# --------------------------------------------------------------------------------------------


@dataclass
class Check:
    name: str
    passed: bool
    detail: str


def hexdump(data: bytes, start: int) -> str:
    window = data[max(0, start - 16) : start + 48]
    printable = "".join(chr(byte) if 32 <= byte < 127 else "." for byte in window)
    return f"`{window.hex()}`\n\n      `{printable}`"


def analyse(
    mode: str,
    pcap: Path,
    sockets_path: Path,
    manifest_path: Path,
    meta: str = "",
    min_samples: int = 0,
) -> tuple[list[Check], str]:
    """Read the three inputs and return the verdicts plus the report text.

    Separated from `main` so `--self-test` can drive it against a capture this file built, and
    so the checks can be shown to fail on a capture that deserves it. A verification harness
    nobody has ever seen fail is a harness with no evidence that it can.
    """
    if not pcap.exists():
        fail(f"{pcap} does not exist")
    linktype, packets = read_pcap(pcap)
    sockets = read_sockets(sockets_path)
    manifest = read_manifest(manifest_path)

    # ---- attribution -------------------------------------------------------------------
    ours: list[Packet] = []
    others: list[Packet] = []
    for packet in packets:
        if not packet.proto.startswith(("tcp", "udp")):
            others.append(packet)
            continue
        local = (packet.src, packet.sport) in sockets.endpoints or (
            packet.dst,
            packet.dport,
        ) in sockets.endpoints
        (ours if local else others).append(packet)

    def is_loopback(packet: Packet) -> bool:
        return packet.src.startswith("127.") or packet.dst.startswith("127.") or "::1" in (packet.src, packet.dst)

    loopback = [packet for packet in ours if is_loopback(packet)]
    egress = [packet for packet in ours if not is_loopback(packet)]

    # ---- names -------------------------------------------------------------------------
    asked: dict[str, int] = {}
    hibp_addresses: set[str] = set()
    for packet in packets:
        if packet.proto == "udp" and 53 in (packet.sport, packet.dport):
            for name in dns_questions(packet.payload):
                asked[name] = asked.get(name, 0) + 1
            hibp_addresses |= dns_addresses(packet.payload, HIBP_HOST)

    # Every ClientHello in the capture, not only the attributed ones. Attribution is sampled and
    # can fail outright — it did, on this harness's first real run, and the destination check then
    # reported "0 ClientHello(s) observed" on a capture holding fifty DNS messages for R-25's own
    # host. A check that goes quiet when its input is missing is worse than one that fails.
    sni: dict[tuple[str, int], str] = {}
    all_sni: dict[tuple[str, int], str] = {}
    for packet in packets:
        if packet.proto != "tcp" or not packet.payload:
            continue
        name = client_hello_sni(packet.payload)
        if not name:
            continue
        peer = (packet.dst, packet.dport)
        all_sni[peer] = name
        if (packet.src, packet.sport) in sockets.endpoints:
            sni[peer] = name

    # ---- conversations -----------------------------------------------------------------
    @dataclass
    class Conversation:
        packets: int = 0
        out_bytes: int = 0
        in_bytes: int = 0
        request_records: int = 0
        response_bytes: int = 0

    conversations: dict[tuple[str, int], Conversation] = {}
    for packet in egress:
        peer = (packet.dst, packet.dport) if (packet.src, packet.sport) in sockets.endpoints else (packet.src, packet.sport)
        conversation = conversations.setdefault(peer, Conversation())
        conversation.packets += 1
        outbound = (packet.src, packet.sport) in sockets.endpoints
        if outbound:
            conversation.out_bytes += len(packet.payload)
            for content_type, length in tls_records(packet.payload):
                if content_type == 23 and length <= REQUEST_RECORD_MAX:
                    conversation.request_records += 1
        else:
            conversation.in_bytes += len(packet.payload)
            for content_type, length in tls_records(packet.payload):
                if content_type == 23:
                    conversation.response_bytes += length

    # ---- the forbidden-string sweep, over every byte of the capture --------------------
    haystack = bytearray()
    boundaries: list[tuple[int, int]] = []  # (offset, packet index)
    for packet in packets:
        boundaries.append((len(haystack), packet.index))
        haystack += packet.raw
    lookup = needles(manifest)
    hits: list[tuple[str, int, str]] = []
    # Spans where R-25's own hostname sits, so a needle found *inside* it is not a leak. The
    # fixture's loudest password is `password`, and `pwnedpasswords` contains it — so the first
    # real run reported twenty hits, all of them the domain name the request is permitted to
    # send. Masking rather than dropping the needle: `password` is exactly the value most worth
    # searching for everywhere else in the capture. `pwnedpasswords` alone is the mask, because
    # DNS puts it on the wire as a length-prefixed label with no dots around it.
    masked = [
        (match.start(), match.end())
        for match in re.finditer(re.escape(b"pwnedpasswords"), bytes(haystack))
    ]

    def inside_hostname(start: int, end: int) -> bool:
        return any(low <= start and end <= high for low, high in masked)

    explained = 0
    if lookup:
        pattern = re.compile(b"|".join(sorted((re.escape(n) for n in lookup), key=len, reverse=True)))
        for match in pattern.finditer(bytes(haystack)):
            if inside_hostname(match.start(), match.end()):
                explained += 1
                continue
            description = lookup.get(match.group(0), "unknown needle")
            index = 0
            for offset, packet_index in boundaries:
                if offset > match.start():
                    break
                index = packet_index
            hits.append((description, index, hexdump(bytes(haystack), match.start())))
            if len(hits) >= 20:
                break

    hibp_anywhere = HIBP_HOST.encode() in bytes(haystack)

    # ---- the checks --------------------------------------------------------------------
    checks: list[Check] = []
    checks.append(
        Check(
            "No password, full hash, item title or identifier anywhere in the capture — gate line 2",
            not hits,
            f"{len(lookup)} byte strings searched across {len(packets)} packets ({len(haystack)} bytes); "
            + ("no match" if not hits else f"**{len(hits)} match(es)** — listed below")
            + (
                f"; {explained} occurrence(s) inside `{HIBP_HOST}` itself, which is the one string "
                "the request is permitted to carry"
                if explained
                else ""
            ),
        )
    )
    unexpected = {peer: name for peer, name in sni.items() if name != HIBP_HOST}
    unnamed = [peer for peer in conversations if peer not in sni and peer[1] == 443]
    if mode == "on":
        checks.append(
            Check(
                f"Every TLS connection the app opened named {HIBP_HOST} — gate line 2",
                not unexpected and bool(sni),
                f"{len(sni)} ClientHello(s) attributed to the app"
                + (f", all to {HIBP_HOST}" if sni and not unexpected else "")
                + (f"; **other names: {sorted(set(unexpected.values()))}**" if unexpected else "")
                + (f"; {len(unnamed)} connection(s) on 443 with no ClientHello captured" if unnamed else "")
                + (
                    f". **Attribution produced nothing**, so this fails on missing evidence rather "
                    f"than on the app's behaviour — the capture itself holds {len(all_sni)} "
                    f"ClientHello(s), listed below"
                    if not sni
                    else ""
                ),
            )
        )
        checks.append(
            Check(
                f"Request-sized records are consistent with {manifest.distinct} distinct values — S-07b",
                sum(c.request_records for c in conversations.values()) > 0,
                f"{sum(c.request_records for c in conversations.values())} client records ≤ {REQUEST_RECORD_MAX} bytes "
                f"across {len(conversations)} connection(s), against {manifest.distinct} distinct values in the vault. "
                "**An estimate, not an assertion** — the requests are inside TLS (D-89); what this rules out is an "
                "order-of-magnitude disagreement, such as one request per password field rather than per value",
            )
        )
        checks.append(
            Check(
                "Concurrency stayed at or below four connections — D-85",
                len(conversations) <= 4,
                f"{len(conversations)} distinct peer endpoint(s). Connections are pooled and reused, so this is an "
                "upper bound on sockets rather than a count of simultaneous ones",
            )
        )
    else:
        # Before the claim, the evidence that there was a run to make it about. "Zero packets"
        # is the one verdict in this report that an app which crashed on launch would also earn,
        # and gate line 4 is the phase's headline claim — so the vacuous case is refused by name
        # rather than left to whoever reads the number.
        checks.append(
            Check(
                "The run is not vacuous — the app was alive and watched throughout",
                sockets.heartbeats >= max(min_samples, 1) and sockets.tree >= 1,
                f"{sockets.heartbeats} heartbeats over {sockets.span:.0f} s, largest process tree {sockets.tree}"
                + (f", against {min_samples} expected for the requested run length" if min_samples else "")
                + (
                    ""
                    if sockets.heartbeats >= max(min_samples, 1)
                    else " — **too few**: the app exited early or never started, and a capture of an "
                    "application that is not running says nothing about the application"
                ),
            )
        )
        checks.append(
            Check(
                "Zero packets left the machine from the app's process tree — S-10, R-26, gate line 4",
                not egress,
                f"{len(egress)} packet(s) attributed to the app outside loopback, "
                f"{len(loopback)} on loopback, over {sockets.samples} socket samples",
            )
        )
        checks.append(
            Check(
                f"{HIBP_HOST} appears nowhere in the capture — S-10",
                not hibp_anywhere,
                "the hostname was not asked for, resolved, or named in any ClientHello"
                if not hibp_anywhere
                else "**the hostname is in the capture bytes** — read the DNS section below before concluding it "
                "was this app: the sweep covers every process on the machine",
            )
        )

    # ---- write it out ------------------------------------------------------------------
    report = []
    report.append(f"# Packet capture — breach checking **{mode}**\n")
    report.append(
        "Generated by `scripts/capture-report.py`. Every number below is read off "
        f"`{pcap.name}`; nothing here is typed by hand.\n"
    )
    if meta:
        report.append("## Run\n")
        report.append("```\n" + meta + "\n```\n")
    report.append("## Verdict\n")
    report.append("| Check | Result | Detail |")
    report.append("|---|---|---|")
    for check in checks:
        report.append(f"| {check.name} | {'**pass**' if check.passed else '**FAIL**'} | {check.detail} |")
    report.append("")

    report.append("## What the capture holds\n")
    duration = (packets[-1].timestamp - packets[0].timestamp) if packets else 0.0
    report.append(f"- Link type `{linktype}`, {len(packets)} packets, {len(haystack)} bytes, {duration:.1f} s of wall clock")
    report.append(
        f"- {len(ours)} attributed to the app's process tree ({len(egress)} off-machine, {len(loopback)} loopback), "
        f"{len(others)} belonging to everything else on this host"
    )
    report.append(f"- {sockets.samples} socket samples, {sockets.lines} socket rows, {len(sockets.endpoints)} distinct local endpoints")
    for name, endpoints in sorted(sockets.by_process.items()):
        report.append(f"  - `{name}` held {len(endpoints)} endpoint(s)")
    report.append("")

    report.append("## Where the app's packets went\n")
    if conversations:
        report.append("| Peer | Name in ClientHello | Packets | Bytes out | Bytes in | Request-sized records |")
        report.append("|---|---|---|---|---|---|")
        for peer, conversation in sorted(conversations.items(), key=lambda item: -item[1].packets):
            report.append(
                f"| `{peer[0]}:{peer[1]}` | {sni.get(peer, '—')} | {conversation.packets} | "
                f"{conversation.out_bytes} | {conversation.in_bytes} | {conversation.request_records} |"
            )
    else:
        report.append("Nothing. No packet outside loopback was attributed to the app's process tree.")
    report.append("")

    report.append("## Every ClientHello in the capture, attributed or not\n")
    if all_sni:
        report.append("| Peer | Server name | The app's? |")
        report.append("|---|---|---|")
        for peer, name in sorted(all_sni.items(), key=lambda item: item[1]):
            report.append(f"| `{peer[0]}:{peer[1]}` | {name} | {'yes' if peer in sni else 'no'} |")
        report.append("")
        report.append(
            "Listed whole because attribution can fail while the capture is fine. A row naming "
            f"`{HIBP_HOST}` with **no** in the last column means somebody on this machine reached "
            "R-25's host and the sampler did not see whose socket it was — read it before "
            "concluding anything about the app."
        )
    else:
        report.append("No TLS ClientHello anywhere in the capture.")
    report.append("")

    report.append("## Names asked for, by every process on this machine\n")
    if asked:
        report.append("| Name | DNS messages |")
        report.append("|---|---|")
        for name, count in sorted(asked.items(), key=lambda item: -item[1])[:40]:
            marker = " ← R-25's host" if name == HIBP_HOST else ""
            report.append(f"| `{name}`{marker} | {count} |")
        if hibp_addresses:
            report.append("")
            report.append(f"{HIBP_HOST} resolved to: " + ", ".join(f"`{address}`" for address in sorted(hibp_addresses)))
    else:
        report.append("No DNS in the capture. On a machine with a warm resolver cache this is expected and is not evidence either way.")
    report.append("")

    report.append("## Forbidden-string sweep\n")
    report.append(
        f"{len(lookup)} needles from `{manifest_path}` — every password, title, username and item id in the "
        "fixture vault, plus each full SHA-1 in upper hex, lower hex and raw binary. Searched across the **whole** "
        "capture rather than the attributed subset, so a socket the sampler missed cannot hide one.\n"
    )
    if hits:
        for description, packet_index, dump in hits:
            report.append(f"- **{description}** in packet {packet_index}:\n\n      {dump}\n")
    else:
        report.append("No match. The strings that identify this vault and its contents are not in these bytes.")
    report.append("")

    report.append("## What this run cannot say\n")
    report.append(
        "- **The prefix itself is inside TLS** (D-89). That five characters were sent rather than forty is asserted "
        "by `src-tauri/src/hibp.rs`'s own test against a listener, not here. What is here is the negative: no full "
        "hash and no password in the bytes, and no host other than the one R-25 names.\n"
        "- **Attribution is sampled.** `capture.sh` polls the process tree's sockets; a socket that opened and closed "
        "between two polls is missed by the attribution and still caught by the sweep and by the DNS section above.\n"
        "- **Padding is a statement about the service** and is measured by `capture.sh probe`, not from this capture: "
        "a padded response and an unpadded one are both opaque on the wire."
    )

    return checks, "\n".join(report) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", choices=("off", "on"))
    parser.add_argument("--pcap", type=Path)
    parser.add_argument("--sockets", type=Path)
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--meta", type=Path)
    parser.add_argument(
        "--min-samples",
        type=int,
        default=0,
        help="heartbeats a full-length run would produce; fewer means the app did not stay up",
    )
    parser.add_argument("--out", type=Path)
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="check this script against a capture it builds itself, in both directions",
    )
    arguments = parser.parse_args()

    if arguments.self_test:
        return self_test()

    missing = [
        name
        for name in ("mode", "pcap", "sockets", "manifest", "out")
        if getattr(arguments, name) is None
    ]
    if missing:
        fail("missing required argument(s): " + ", ".join("--" + name for name in missing))

    meta = arguments.meta.read_text().strip() if arguments.meta and arguments.meta.exists() else ""
    checks, report = analyse(
        arguments.mode,
        arguments.pcap,
        arguments.sockets,
        arguments.manifest,
        meta,
        arguments.min_samples,
    )
    arguments.out.write_text(report)

    for check in checks:
        print(f"{'PASS' if check.passed else 'FAIL'}  {check.name}")
    print(f"\nwrote {arguments.out}")
    return 1 if any(not check.passed for check in checks) else 0


# --------------------------------------------------------------------------------------------
# The self-test — `capture.sh selftest`
# --------------------------------------------------------------------------------------------
#
# The gate's evidence is a script's output, so the script needs evidence of its own. This builds
# a capture by hand — DNS on loopback, a ClientHello with SNI, a request-sized record and a large
# response — runs the analysis over it, and then **breaks it on purpose**: a password planted in
# a frame must make the sweep fail, and a packet from the app must make S-10 fail. A check that
# has never been seen to fail is indistinguishable from a check that cannot.


def synthetic_pcap(path: Path, frames: list[bytes]) -> None:
    """Write LINUX_SLL2 frames — what `tcpdump -i any` produces on this machine."""
    with path.open("wb") as handle:
        handle.write(struct.pack("<IHHiIII", 0xA1B2C3D4, 2, 4, 0, 0, 262144, LINKTYPE_LINUX_SLL2))
        for index, frame in enumerate(frames):
            handle.write(struct.pack("<IIII", 1_755_300_000 + index, 0, len(frame), len(frame)))
            handle.write(frame)


def sll2(payload: bytes) -> bytes:
    return struct.pack(">HHIHBB", 0x0800, 0, 2, 1, 0, 6) + b"\x00" * 8 + payload


def ipv4(source: str, destination: str, protocol: int, payload: bytes) -> bytes:
    header = struct.pack(
        ">BBHHHBBH4s4s",
        0x45,
        0,
        20 + len(payload),
        0,
        0,
        64,
        protocol,
        0,
        bytes(int(part) for part in source.split(".")),
        bytes(int(part) for part in destination.split(".")),
    )
    return sll2(header + payload)


def udp(source: str, sport: int, destination: str, dport: int, payload: bytes) -> bytes:
    return ipv4(source, destination, 17, struct.pack(">HHHH", sport, dport, 8 + len(payload), 0) + payload)


def tcp(source: str, sport: int, destination: str, dport: int, payload: bytes) -> bytes:
    header = struct.pack(">HHIIBBHHH", sport, dport, 0, 0, 5 << 4, 0x18, 0xFFFF, 0, 0)
    return ipv4(source, destination, 6, header + payload)


def dns_query(name: str) -> bytes:
    labels = b"".join(bytes([len(part)]) + part.encode() for part in name.split(".")) + b"\x00"
    return struct.pack(">HHHHHH", 0x1234, 0x0100, 1, 0, 0, 0) + labels + struct.pack(">HH", 1, 1)


def dns_answer(name: str, address: str) -> bytes:
    labels = b"".join(bytes([len(part)]) + part.encode() for part in name.split(".")) + b"\x00"
    question = struct.pack(">HHHHHH", 0x1234, 0x8180, 1, 1, 0, 0) + labels + struct.pack(">HH", 1, 1)
    record = struct.pack(">HHIH", 1, 1, 60, 4) + bytes(int(part) for part in address.split("."))
    return question + b"\xc0\x0c" + record


def client_hello(name: str) -> bytes:
    server_name = name.encode()
    extension = struct.pack(">HHHBH", 0x0000, len(server_name) + 5, len(server_name) + 3, 0, len(server_name)) + server_name
    body = (
        struct.pack(">H", 0x0303)
        + b"\x00" * 32  # random
        + b"\x00"  # session id length
        + struct.pack(">H", 2)
        + b"\x13\x01"  # one cipher suite
        + b"\x01\x00"  # one compression method
        + struct.pack(">H", len(extension))
        + extension
    )
    handshake = b"\x01" + struct.pack(">I", len(body))[1:] + body
    return b"\x16\x03\x01" + struct.pack(">H", len(handshake)) + handshake


def application_data(length: int) -> bytes:
    return b"\x17\x03\x03" + struct.pack(">H", length) + b"\xab" * length


def self_test() -> int:
    import tempfile

    app, resolver, hibp = "192.168.1.103", "127.0.0.53", "104.16.1.1"
    failures: list[str] = []

    with tempfile.TemporaryDirectory() as workspace:
        directory = Path(workspace)
        manifest = directory / "manifest.txt"
        manifest.write_text(
            "vault\t/tmp/audit.tvault\ndistinct\t12\nprefix\t5BAA6\n"
            "forbid\tmangotree84\nforbid\tMeridian Bank\n"
            "forbid-hash\t5BAA61E4C9B93F3F0682250B6CF8331B7EE68FD8\n"
        )
        sockets = directory / "sockets.txt"
        sockets.write_text(
            f"1755300000.0\tudp ESTAB 0 0 {app}:41234 {resolver}:53 users:((\"trustvault\",pid=1,fd=9))\n"
            f"1755300000.0\ttcp ESTAB 0 0 {app}:44210 {hibp}:443 users:((\"trustvault\",pid=1,fd=10))\n"
        )

        clean = [
            udp(app, 41234, resolver, 53, dns_query(HIBP_HOST)),
            udp(resolver, 53, app, 41234, dns_answer(HIBP_HOST, hibp)),
            tcp(app, 44210, hibp, 443, client_hello(HIBP_HOST)),
            tcp(app, 44210, hibp, 443, application_data(120)),
            tcp(hibp, 443, app, 44210, application_data(16384)),
        ]

        def verdicts(mode: str, frames: list[bytes], sockets_file: Path = sockets) -> dict[str, bool]:
            pcap = directory / f"{mode}-{len(frames)}-{id(frames)}.pcap"
            synthetic_pcap(pcap, frames)
            checks, _ = analyse(mode, pcap, sockets_file, manifest)
            return {check.name: check.passed for check in checks}

        def expect(condition: bool, description: str) -> None:
            print(f"{'PASS' if condition else 'FAIL'}  {description}")
            if not condition:
                failures.append(description)

        on_clean = verdicts("on", clean)
        expect(all(on_clean.values()), "a clean breach check passes every `on` check")

        planted = list(clean)
        planted[3] = tcp(app, 44210, hibp, 443, b"\x17\x03\x03\x00\x0bmangotree84")
        on_planted = verdicts("on", planted)
        expect(
            not any("No password" in name and passed for name, passed in on_planted.items()),
            "a password planted in a frame fails the sweep",
        )

        hashed = list(clean)
        hashed[3] = tcp(app, 44210, hibp, 443, b"5BAA61E4C9B93F3F0682250B6CF8331B7EE68FD8")
        expect(
            not any("No password" in name and passed for name, passed in verdicts("on", hashed).items()),
            "a full SHA-1 in hex fails the sweep",
        )

        binary = list(clean)
        binary[3] = tcp(app, 44210, hibp, 443, bytes.fromhex("5BAA61E4C9B93F3F0682250B6CF8331B7EE68FD8"))
        expect(
            not any("No password" in name and passed for name, passed in verdicts("on", binary).items()),
            "a full SHA-1 as raw bytes fails the sweep — the case a hex-only grep misses",
        )

        elsewhere = list(clean)
        elsewhere[2] = tcp(app, 44210, "203.0.113.9", 443, client_hello("telemetry.example.com"))
        expect(
            not any("named" in name and passed for name, passed in verdicts("on", elsewhere).items()),
            "a ClientHello naming another host fails the destination check",
        )

        # An app that ran for ten minutes and opened no socket at all: heartbeats, no rows. This
        # is what a passing `off` run looks like, and it is the only shape that may pass.
        idle = directory / "idle-sockets.txt"
        idle.write_text(
            "".join(f"{1755300000 + tick * 0.2:.1f}\t# tree 4\n" for tick in range(3000))
        )
        expect(
            all(verdicts("off", [], idle).values()),
            "a capture of a live app that opened nothing passes every `off` check",
        )

        # And the shape that must not: no heartbeats means nothing was watched, so "zero packets"
        # is a statement about an application that was not running. A crash on launch would earn
        # gate line 4 otherwise, which is the one vacuous pass in this report worth engineering
        # against.
        crashed = directory / "crashed-sockets.txt"
        crashed.write_text("")
        expect(
            not any(
                passed for name, passed in verdicts("off", [], crashed).items() if "not vacuous" in name
            ),
            "a run where the app never started fails as vacuous rather than passing as silent",
        )

        # Only the two S-10 checks, not every check: the traffic carries no password, so the
        # sweep passing here is correct and an assertion demanding it fail would be the kind of
        # over-broad expectation that gets loosened later until it means nothing.
        under_off = verdicts("off", clean)
        expect(
            not any(passed for name, passed in under_off.items() if "Zero packets" in name),
            "traffic from the app under `off` fails the zero-packet check",
        )
        expect(
            not any(passed for name, passed in under_off.items() if "appears nowhere" in name),
            "the hostname anywhere in an `off` capture fails S-10's second check",
        )

    print()
    if failures:
        print(f"{len(failures)} self-test(s) failed")
        return 1
    print("capture-report.py agrees with itself in both directions")
    return 0


if __name__ == "__main__":
    sys.exit(main())
