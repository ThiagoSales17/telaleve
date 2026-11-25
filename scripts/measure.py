#!/usr/bin/env python3
"""CSV de RSS somada e CPU da árvore do emissor; não mede atraso entre telas."""
import argparse
import csv
import os
import pathlib
import time


def processes(root):
    snapshot = {}
    for entry in pathlib.Path('/proc').iterdir():
        if not entry.name.isdigit():
            continue
        try:
            raw = (entry / 'stat').read_text()
            fields = raw[raw.rfind(')') + 2:].split()
            snapshot[int(entry.name)] = (int(fields[1]), int(fields[11]) + int(fields[12]), int(fields[21]))
        except (OSError, ValueError, IndexError):
            pass
    selected = {root}
    while True:
        children = {pid for pid, (parent, _, _) in snapshot.items() if parent in selected}
        if children <= selected:
            break
        selected |= children
    return {pid: snapshot[pid] for pid in selected if pid in snapshot}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('pid', type=int)
    parser.add_argument('--seconds', type=int, default=1800)
    parser.add_argument('--output', default='measurement.csv')
    args = parser.parse_args()
    if args.pid <= 0 or args.seconds <= 0:
        parser.error('PID e duração devem ser positivos')
    hz, page = os.sysconf('SC_CLK_TCK'), os.sysconf('SC_PAGE_SIZE')
    started = previous_time = time.monotonic()
    previous = processes(args.pid)
    if args.pid not in previous:
        parser.error('Processo indisponível')
    with open(args.output, 'w', newline='') as output:
        writer = csv.writer(output)
        writer.writerow(['elapsed_s', 'processes', 'rss_mib_sum', 'cpu_percent_one_core'])
        while time.monotonic() - started < args.seconds:
            time.sleep(1)
            current_time, current = time.monotonic(), processes(args.pid)
            if args.pid not in current:
                break
            ticks = sum(max(0, values[1] - previous[pid][1]) for pid, values in current.items() if pid in previous)
            cpu = ticks / hz / (current_time - previous_time) * 100
            rss = sum(values[2] for values in current.values()) * page / 1024**2
            writer.writerow([round(current_time - started, 2), len(current), round(rss, 2), round(cpu, 2)])
            output.flush()
            previous, previous_time = current, current_time

if __name__ == '__main__':
    main()
