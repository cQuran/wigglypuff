# WigglIO
**`API Documentation`** |
------------------- |
[![Documentation](https://img.shields.io/badge/api-documentation-pink.svg)](http://bit.ly/WigglypuffAPI) |

WigglIO is an end-to-end webrtc Signalling & Audio Media Server service. Wigglypuff was designed for flexibility and modularity. The bulk of the API layer is implemented in Rust, making it super fast to modify and extend with new functionality.
</br>
</br>
Wigglypuff is built on the belief that the WebRTC standard would transcend the pure browser environment and that native apps, implementing the same protocols and API's, would become an important part of the WebRTC ecosystem. This is especially true on mobile platforms where native app distribution is often preferred over pure web apps. Wigglypuff provide the WebRTC-backend to web browsers.
</br>
</br>
Having independent, interoperable, implementations is important for the health of any standard, and WebRTC is no exception. The ambition of WigglIO is to follow the WebRTC standard closely as it continues to evolve.

# Run
To run the development mode, we recommend to use Debian Slim Buster:
```
$ docker build -t wigglypuff-dev -f docker/slim-buster/Dockerfile .
$ docker run --rm -p 6030:6030 -e HOST=0.0.0.0 -e PORT=6030 -it 
wigglypuff-dev cargo run
```
To build the release mode, we recommend to use Alpine with musl build:
```
$ docker build -t wigglypuff -f docker/alpine/Dockerfile .
$ docker run --rm -p 6030:6030 -e HOST=0.0.0.0 -e PORT=6030 -it 
wigglypuff
```

# Audio Media server
![arch](assets/routing-algorithm.png)

# Mobile client flow
![arch](assets/mobile.png)
## Contribution guidelines

**If you want to contribute to WigglIO, be sure to review the
[contribution guidelines](CONTRIBUTING.md). This project adheres to Wigglypuff's
[code of conduct](CODE_OF_CONDUCT.md). By participating, you are expected to
uphold this code.**

**We use [GitHub issues](https://github.com/cQuran/wigglypuff/issues) for
tracking requests and bugs, please see
[Wigglypuff Discuss](https://cquran.slack.com/apps)
for general questions and discussion, and please direct specific questions to
[Slack Channel](https://cquran.slack.com/apps).**

**We appreciate all contributions. If you are planning to contribute back bug-fixes, please do so without any further discussion.**

**If you plan to contribute new features, utility functions, or extensions to the core, please first open an issue and discuss the feature with us. Sending a PR without discussion might end up resulting in a rejected PR because we might be taking the core in a different direction than you might be aware of.**

