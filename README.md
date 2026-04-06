# alfred-slack-2-markdown-rust

> Slack emojis to markdown (Rust version)

An alfred plugin that allows you to use your custom slack emojis from your organization in markdown.

This is a Rust rewrite of the original [alfred-slack-2-markdown](https://github.com/scottlnorvell/alfred-slack-2-markdown).

## Installation

1. Download the `alfred-slack-2-markdown.alfredworkflow` file from the [latest release](https://github.com/scottlnorvell/alfred-slack-2-markdown-rust/releases).
2. Double click to install.

### Build from source

Requires [Rust](https://www.rust-lang.org/tools/install).

```bash
cargo build --release
cp target/release/alfred-slack-2-markdown-rust .
```

## Configuration

- Obtain an api token for your slack group with the `emoji:read` permissions. (see [here](https://api.slack.com/methods/emoji.list))
- In alfred type `smdc`, <kbd>Space</kbd>, and then paste in your slack api key.

## Usage

In Alfred, type `smd`, <kbd>Space</kbd>, and then start searching emojis to copy as markdown.
In Alfred, type `smc`, <kbd>Space</kbd>, and then start searching emojis to copy (and paste) the actual image file (preserves GIF animation).


## Troubleshooting
### I can't see the pictures 😿
For some reason, alfred can only display images from your computer.
There is an initial setup to download all of the current emojis, and
you will periodically need to download all of the _new_ emojis that your team adds. To do this:

- In Alfred, type `smdu`, <kbd>Space</kbd>
- You'll see a huge screen saying `UPDATED`
- That's it!

> NOTE: it takes kind of a long time the first time you do it. (assuming you have a lot of emojis in your org)

## License

MIT © [Scott Norvell](http://scottlnorvell.com)
