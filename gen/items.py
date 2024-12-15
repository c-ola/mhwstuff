import json
from dataclasses import dataclass
from typing import Dict, List

@dataclass
class Item:
    name: str
    explanation: str
    buy_price: int
    sell_price: int
    item_type: str
    icon_color: str
    rarity: str

@dataclass
class ItemRecipe:
    name: str
    explanation: str
    buy_price: int
    sell_price: int

