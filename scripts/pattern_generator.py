# This may be useless, since the test patterns should be hard-coded into the test file...
import json


class Entity:
    def __init__(self, ID: int, signature: str):
        self.ID = ID
        self.signature = signature


class Event:
    def __init__(
        self, ID: int, signature: str, SubjectID: int, ObjectID: int, Parents: list[int]
    ):
        self.ID = ID
        self.signature = signature
        self.SubjectID = SubjectID
        self.ObjectID = ObjectID
        self.Parents = Parents


def pattern_from_data(
    entities: list[Entity],
    events: list[Event],
    version: str = "0.2.0",
    useRegex: bool = True,
):
    pattern = dict()
    pattern["Version"] = version
    pattern["UseRegex"] = useRegex

    entities_list = list()
    for entity in entities:
        entity_dict = dict()
        entity_dict["ID"] = entity.ID
        entity_dict["Signature"] = entity.signature

        entities_list.append(entity_dict)

    events_list = list()
    for event in events:
        event_dict = dict()
        event_dict["ID"] = event.ID
        event_dict["Signature"] = event.signature
        event_dict["SubjectID"] = event.SubjectID
        event_dict["ObjectID"] = event.ObjectID
        event_dict["Parents"] = event.Parents

        events_list.append(event_dict)

    pattern["Entities"] = entities_list
    pattern["Events"] = events_list
    pattern_json = json.dumps(pattern, indent=4)
    print(pattern_json)

    with open("/tmp/test_pattern_single_line.json", "w") as file:
        json.dump(pattern, file)


if __name__ == "__main__":
    entities = [
        Entity(0, "0"),
        Entity(1, "1"),
        Entity(2, "2"),
        Entity(3, "3"),
        Entity(4, "4"),
        Entity(5, "5"),
        Entity(6, "6"),
        Entity(7, "7"),
        Entity(8, "8"),
    ]

    events = [
        Event(0, "0", 0, 1, []),
        Event(1, "1", 3, 4, [0]),
        Event(2, "2", 7, 8, [0]),
        Event(3, "3", 5, 2, [0]),
        Event(4, "4", 4, 5, [1]),
        Event(5, "5", 2, 6, [3]),
        Event(6, "6", 5, 7, [2]),
        Event(7, "7", 1, 2, [0]),
    ]

    pattern_from_data(entities, events)
