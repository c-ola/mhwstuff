import json
from dataclasses import dataclass
from typing import Dict, List

f = open("outputs/skills/skillcommondata.user.3.json")
skill_common_data = json.load(f)

f = open("outputs/msg/skillcommon.msg.23")
skill_common_msg_data = json.load(f)

f = open("outputs/skills/skilldata.user.3.json")
skill_data = json.load(f)

f = open("outputs/msg/skill.msg.23")
skill_msg_data = json.load(f)

@dataclass
class Skill:
    name: str
    explain: str
    levels: Dict



skills = {}
for s in skill_common_data['values']:
    
    name = skill_common_msg_data.get(s['skill_name'])
    explain = skill_common_msg_data.get(s['skill_explain'])
    if name is None or explain is None:
        continue
    id = s['skill_id']
    if id != "NONE":
        skills[id] = Skill(name['content'][1], explain['content'][1], {})

for s in skill_data['values']:
    name = skill_msg_data.get(s['skill_name'])
    level = s['skill_lv']
    explain = skill_msg_data.get(s['skill_explain'])
    if name is None or explain is None:
        continue
    id = s['skill_id']
    if skills.get(id) is not None:
        skill = skills[id]
        skill.levels[level] = explain['content'][1]

page = """
{{NavigationMHWilds}}
{| class="wikitable sortable wide"
! colspan=3 | '''List of Armor Skills'''
|-
! Name
! Description
! Effects/levels
"""

for (_, s) in skills.items():
    page += f"|-\n"
    page += f"| rowspan=\"1\" | {s.name}\n"
    page += f"|{s.explain.replace('\r\n', ' ')}\n"
    page += f"|\n"
    for l in s.levels.items():
        page += f"# {l[1].replace('\r\n', ' ')}\n"
page += '|-\n'
page += "|}\n"
print(page)

