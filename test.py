import json
from collections import OrderedDict

settings_file = open("bot_settings.json")
settings = json.load(settings_file, object_pairs_hook=OrderedDict)

for key in settings["attribute_names"].keys():
    print(key)