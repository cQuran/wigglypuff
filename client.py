import asyncio
import websockets
import json
import sys


async def hello(uri):
    async with websockets.connect(uri) as websocket:

        # await websocket.send(json.dumps({"action": "ClickAya", "aya": 1}))
        await websocket.send(json.dumps({"action": "OfferCorrection", "uuid": sys.argv[1]}))
        print("SENT")
        while True:
            a = await websocket.recv()
            print(a)


asyncio.get_event_loop().run_until_complete(
    hello("ws://0.0.0.0:6040/api/room/join/dssn/" + sys.argv[1])
)
