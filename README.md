# Twitch Hunter

Twitch Hunter is a tool that connects to multiple Twitch IRC channels and looks for specific regex patterns in chat messages. It uses the Twitch Helix API to fetch streams from a specific category and then monitors the chat messages in those streams.

## Features

- Connects to multiple Twitch IRC channels.
- Filters chat messages based on a regex pattern.
- Uses the Twitch Helix API to fetch streams from a specific category.

## Installation

Using cargo:

```
  cargo install twitch-hunter
```

Or build from source:

1. Clone the repository:

```
  git clone https://github.com/ayoubdya/twitch-hunter.git
  cd twitch-hunter
```

2. Install dependencies:

```
  cargo build --release
```

## Usage

Run the application:

```
Usage: twitch-hunter [OPTIONS] --client-id <CLIENT_ID> --access-token <ACCESS_TOKEN> --category-name <CATEGORY_NAME> --filter <REGEX>

Options:
      --client-id <CLIENT_ID>
      --access-token <ACCESS_TOKEN>
  -c, --category-name <CATEGORY_NAME>
  -b, --batch-size <BATCH_SIZE>        [default: 600]
  -f, --filter <REGEX>
  -h, --help                           Print help
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## Authors

- Ayoub DYA - [ayoubdya@gmail.com](mailto:ayoubdya@gmail.com)
