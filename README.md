# Vis
Console audio visualizer with Youtube API

You can see it in action in my [video](https://www.youtube.com/watch?v=gg5kk1sIKPg)

## What does it do?
You can input video title and it will download it from Youtube and then play it with cool visuals

You will need to modify this line:
```rust
const API_KEY: &str = /* your API key */;
```

## Commands
* `p` followed by title clears the queue and plays that title
* `q` followed by title adds that title to queue
* `s` skip current song in queue

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