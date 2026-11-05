#!/usr/bin/env python3
"""Analyze latest USB frames and persist bounded clothing-color events in camera telemetry."""
import argparse
import json
from pathlib import Path
import runpy
import time
import uuid

HELPERS = runpy.run_path(str(Path(__file__).with_name('create-rtsp-dashboard.py')))


def new_events(persons, timestamp, recent):
    events = []
    previous = dict(recent)
    for index, person in enumerate(persons):
        signature = (person['upper']['color'], person['lower']['color'])
        # Suppress repeated color combinations for 10 seconds; this is not identity tracking.
        if timestamp - previous.get(signature, float('-inf')) < 10:
            continue
        recent[signature] = timestamp
        events.append({'id': str(uuid.uuid4()), 'timestamp': timestamp, 'person_number': index + 1,
                       'upper': person['upper'], 'lower': person['lower'], 'bbox': person['bbox']})
    return events


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--server', default='http://127.0.0.1:9375')
    parser.add_argument('--device-id', default='cam2-rtsp')
    parser.add_argument('--interval', type=float, default=5)
    parser.add_argument('--token-file', type=Path, required=True)
    args = parser.parse_args()
    if not 1 <= args.interval <= 60:
        parser.error("--interval must be between 1 and 60 seconds")
    token = args.token_file.read_text().strip()
    request = HELPERS['request']
    events, recent, last_frame = [], {}, None
    last_status = None
    count = 0

    def metric(name, value):
        request(args.server, f'/devices/{args.device_id}/metrics', token, {'metric': name, 'value': value})

    try:
        current = request(args.server, f'/devices/{args.device_id}/current', token)['metrics']
        events = json.loads(current.get('clothing_events', {}).get('value', '[]'))[:100]
        for event in reversed(events):
            recent[(event['upper']['color'], event['lower']['color'])] = event['timestamp']
    except (OSError, ValueError, KeyError, TypeError):
        pass
    while True:
        started = time.monotonic()
        try:
            current = request(args.server, f'/devices/{args.device_id}/current', token)['metrics']
            image_metric = current.get('image', {})
            image = image_metric.get('value')
            frame = (image_metric.get('timestamp'), image)
            if not isinstance(image, str) or not image.startswith('data:image/') or frame == last_frame:
                time.sleep(1)
                continue
            last_frame = frame
            result = request(args.server, '/extensions/clothing-colors/command', token,
                             {'command': 'analyze', 'args': {'image_base64': image, 'max_people': 2,
                                 'frame_id': str(uuid.uuid4()), 'publish_live': True,
                                 'device_id': args.device_id, 'initial_events': events}})
            events = []  # The worker now owns and persists event history.
            if last_status != 'running':
                metric('clothing_status', 'running')
                last_status = 'running'
            count += 1
            if count == 1 or count % 12 == 0:
                print(f"Analyzed frame: {len(result['persons'])} people, {result.get('new_events', 0)} events, {result['processing_ms']} ms", flush=True)
        except Exception as error:
            print(f'Analysis retry: {type(error).__name__}', flush=True)
            try:
                if last_status != 'error':
                    metric('clothing_status', 'error')
                    last_status = 'error'
            except Exception:
                pass
        time.sleep(max(1.0, args.interval - (time.monotonic() - started)))


if __name__ == '__main__':
    main()
