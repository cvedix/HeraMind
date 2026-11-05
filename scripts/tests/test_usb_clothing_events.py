import runpy
from pathlib import Path
import unittest

EVENTS = runpy.run_path(str(Path(__file__).resolve().parents[1] / "usb-clothing-events.py"))


class ClothingEventTests(unittest.TestCase):
    def test_repeated_colors_are_suppressed_and_reappear_after_cooldown(self):
        person = {"upper": {"color": "blue"}, "lower": {"color": "black"}, "bbox": {}}
        recent = {}
        create = EVENTS["new_events"]
        first = create([person], 100, recent)
        self.assertEqual(len(first), 1)
        self.assertEqual(first[0]["person_number"], 1)
        self.assertEqual(create([person], 109, recent), [])
        self.assertEqual(len(create([person], 110, recent)), 1)
        person["upper"] = {"color": "red"}
        self.assertEqual(len(create([person], 111, recent)), 1)

    def test_two_people_with_same_colors_in_one_frame_have_two_events(self):
        person = {"upper": {"color": "blue"}, "lower": {"color": "black"}, "bbox": {}}
        events = EVENTS["new_events"]([person, person], 100, {})
        self.assertEqual([event["person_number"] for event in events], [1, 2])

    def test_empty_frame_does_not_create_events(self):
        self.assertEqual(EVENTS["new_events"]([], 100, {}), [])


if __name__ == "__main__":
    unittest.main()
