#!/usr/bin/env python3
"""
alerts.toml yönetim aracı — shell'den alarm ekleme/silme/güncelleme.

Kullanım:
  python3 scripts/alerts_cli.py list
  python3 scripts/alerts_cli.py add --symbol HEIUSDT --condition above --price 0.22 --voice "..." [--cooldown 30] [--tolerance 0.0005]
  python3 scripts/alerts_cli.py update --symbol HEIUSDT --condition above --old-price 0.21628 --price 0.22 [--voice "..."] [--cooldown 30]
  python3 scripts/alerts_cli.py remove --symbol HEIUSDT --condition above --price 0.21628
"""

import argparse
import os
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
CONFIG = os.environ.get("ALERT_CONFIG", os.path.join(ROOT, "alerts.toml"))


def parse_config(path):
    """alerts.toml'u bloklar halinde ayrıştırır: (header_satırları, [blok_metinleri])"""
    if not os.path.exists(path):
        return [], []
    with open(path) as f:
        lines = f.read().splitlines()
    header = []
    blocks = []
    current = []
    in_block = False
    for line in lines:
        stripped = line.strip()
        if stripped.startswith("[[alerts]]"):
            if current:
                blocks.append("\n".join(current))
            current = [line]
            in_block = True
        elif in_block:
            if stripped.startswith("[["):
                blocks.append("\n".join(current))
                current = [line]
            else:
                current.append(line)
        else:
            header.append(line)
    if current:
        blocks.append("\n".join(current))
    return header, blocks


def parse_block(block):
    """Blok metninden alanları sözlük olarak çıkarır (değerler temizlenir)."""
    data = {"_lines": block.splitlines()}
    for line in block.splitlines():
        s = line.strip()
        if s.startswith("[[") or s.startswith("#"):
            continue
        if "=" in s:
            key, _, val = s.partition("=")
            data[key.strip()] = val.strip().strip('"').strip("'")
    return data


def _norm_price(v):
    """'70.0' ve '70' aynı sayıya normalize edilir."""
    if v is None:
        return None
    try:
        return str(float(str(v)))
    except Exception:
        return str(v).strip()


def block_key(block):
    d = parse_block(block)
    return (d.get("symbol"), d.get("condition"), _norm_price(d.get("price")))


def fmt_price(v):
    try:
        f = float(v)
        return repr(f)
    except Exception:
        return v


def render_block(d):
    lines = ["[[alerts]]"]
    if "symbol" in d:
        lines.append(f'symbol = "{d["symbol"]}"')
    if "condition" in d:
        lines.append(f'condition = "{d["condition"]}"')
    if "price" in d:
        lines.append(f"price = {fmt_price(d['price'])}")
    if "tolerance_pct" in d:
        lines.append(f"tolerance_pct = {fmt_price(d['tolerance_pct'])}")
    if "voice" in d:
        lines.append(f'voice = "{d["voice"]}"')
    if "cooldown_sec" in d:
        lines.append(f"cooldown_sec = {d['cooldown_sec']}")
    return "\n".join(lines)


def write_config(path, header, blocks):
    out = "\n".join(header)
    if out and blocks:
        out += "\n"
    if blocks:
        out += "\n".join(blocks)
        out += "\n"
    with open(path, "w") as f:
        f.write(out)


def cmd_list():
    header, blocks = parse_config(CONFIG)
    if not blocks:
        print("  📭 Alarmsız (alerts.toml'da blok yok)")
        return
    for i, b in enumerate(blocks, 1):
        d = parse_block(b)
        voice = d.get("voice", "").strip('"')
        print(f"  [{i}] {d.get('symbol','?'):<9} {d.get('condition','?'):<6} "
              f"fiyat={d.get('price','?'):<10} tol={d.get('tolerance_pct','-')} "
              f"cooldown={d.get('cooldown_sec','-')}s "
              f"{'🗣️ ' + voice if voice else '🔊 beep'}")


def cmd_add(args):
    header, blocks = parse_config(CONFIG)
    d = {
        "symbol": args.symbol.upper(),
        "condition": args.condition.lower(),
        "price": args.price,
    }
    if args.tolerance:
        d["tolerance_pct"] = args.tolerance
    if args.voice:
        d["voice"] = args.voice
    d["cooldown_sec"] = args.cooldown
    blocks.append(render_block(d))
    write_config(CONFIG, header, blocks)
    print(f"✅ Eklendi: {d['symbol']} {d['condition']} {d['price']}")


def cmd_update(args):
    header, blocks = parse_config(CONFIG)
    old_key = (args.symbol.upper(), args.condition.lower(), _norm_price(args.old_price))
    found = False
    for i, b in enumerate(blocks):
        if block_key(b) == old_key:
            d = parse_block(b)
            d["_lines"] = None
            if args.price:
                d["price"] = args.price
            if args.voice is not None:
                d["voice"] = args.voice
            if args.cooldown:
                d["cooldown_sec"] = args.cooldown
            if args.tolerance:
                d["tolerance_pct"] = args.tolerance
            blocks[i] = render_block(d)
            found = True
            break
    if not found:
        print(f"❌ Alarm bulunamadı: {old_key[0]} {old_key[1]} {old_key[2]}")
        sys.exit(1)
    write_config(CONFIG, header, blocks)
    print(f"✅ Güncellendi: {old_key[0]} {old_key[1]}")


def cmd_remove(args):
    header, blocks = parse_config(CONFIG)
    target = (args.symbol.upper(), args.condition.lower(), _norm_price(args.price))
    before = len(blocks)
    blocks = [b for b in blocks if block_key(b) != target]
    if len(blocks) == before:
        print(f"❌ Alarm bulunamadı: {target[0]} {target[1]} {target[2]}")
        sys.exit(1)
    write_config(CONFIG, header, blocks)
    print(f"✅ Silindi: {target[0]} {target[1]} {target[2]}")


def main():
    p = argparse.ArgumentParser(description="alerts.toml yönetimi")
    sub = p.add_subparsers(dest="cmd", required=True)

    sub.add_parser("list", help="alarmları listele")

    a = sub.add_parser("add")
    a.add_argument("--symbol", required=True)
    a.add_argument("--condition", required=True, choices=["above", "below", "cross", "touch"])
    a.add_argument("--price", required=True)
    a.add_argument("--voice")
    a.add_argument("--cooldown", default=30)
    a.add_argument("--tolerance")

    u = sub.add_parser("update")
    u.add_argument("--symbol", required=True)
    u.add_argument("--condition", required=True, choices=["above", "below", "cross", "touch"])
    u.add_argument("--old-price", required=True)
    u.add_argument("--price")
    u.add_argument("--voice")
    u.add_argument("--cooldown")
    u.add_argument("--tolerance")

    r = sub.add_parser("remove")
    r.add_argument("--symbol", required=True)
    r.add_argument("--condition", required=True, choices=["above", "below", "cross", "touch"])
    r.add_argument("--price", required=True)

    args = p.parse_args()
    if args.cmd == "list":
        cmd_list()
    elif args.cmd == "add":
        cmd_add(args)
    elif args.cmd == "update":
        cmd_update(args)
    elif args.cmd == "remove":
        cmd_remove(args)


if __name__ == "__main__":
    main()
