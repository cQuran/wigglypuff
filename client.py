import asyncio
import websockets
import json
import sys

async def room_master(uri):
    async with websockets.connect(uri) as websocket:
        await websocket.send(json.dumps({"action": "ClickAya", "aya": 1}))
        await websocket.send(json.dumps({"action": "MuteAllUser"}))
        await websocket.send(json.dumps({"action": "MuteUser", "uuid": "abdan"}))
        await websocket.send(json.dumps({"action": "MoveSura", "id_quran": 1}))
        while True:
            message = await websocket.recv()
            print(message)
            message_json = json.loads(message)
            if message_json['action'] == "OfferCorrection":
                message_json['action'] = "AnswerCorrection"
                message_json['result'] = True
                await websocket.send(json.dumps(message_json))
                

async def client(uri):
    async with websockets.connect(uri) as websocket:
        await websocket.send(json.dumps({"action": "OfferCorrection", "uuid": "abdan"}))
        while True:
            message = await websocket.recv()
            print(message)

if sys.argv[1] == "room_master":
    asyncio.get_event_loop().run_until_complete(
        room_master("ws://0.0.0.0:6040/api/room/join/dssn/" + sys.argv[2])
    )
elif sys.argv[1] == "user":
    asyncio.get_event_loop().run_until_complete(
        client("ws://0.0.0.0:6040/api/room/join/dssn/" + sys.argv[2])
    )