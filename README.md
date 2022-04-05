# Vis
Console audio visualizer with Youtube API

## What does it do?
You can input video title and it will download it from Youtube and then play it with cool visuals

You will need to modify this line:
```rust
const API_KEY: &str = /* your API key */;
```

## Prerequisites
* Your own [Youtube API](https://developers.google.com/youtube/v3/getting-started) key
* [yt-dlp](https://github.com/yt-dlp/yt-dlp) for downloading youtube videos
* [ffmpeg](https://ffmpeg.org) for parsing between audio formats

## Todo
This app is still missing some features like:
* streaming and visualizing in real-time
* support for resizing console window during runtime
* alternative way of visualizing
* more quality of life commands