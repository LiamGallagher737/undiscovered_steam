<div align="center">

# Undiscovered Steam

A CLI for finding Steam games nobody has heard of

![Demo](demo.gif)

[<img alt="Github Repository" src="https://img.shields.io/badge/github-undiscovered_steam-8da0cb?style=for-the-badge&labelColor=555555&logo=github">](https://github.com/LiamGallagher737/undiscovered_steam)
[<img alt="GitHub Workflow Status" src="https://img.shields.io/github/actions/workflow/status/LiamGallagher737/undiscovered_steam/ci.yml?branch=main&style=for-the-badge">](https://github.com/LiamGallagher737/undiscovered_steam/actions/workflows/ci.yml)

</div>

## How it works

The program uses the Steam web API to preform a search with randomly selected word. Another request is then preformed for the full app data for each of the results. These results are then filtered according to your selected options.

## Options

These are the current options you can filter by

- Max Price
- Max Review Count
- Supported Platforms

## Keybinds

| Key              | Description           |
|------------------|-----------------------|
| <kbd>↑/↓</kbd>   | Move selection        |
| <kbd>Enter</kbd> | Proceed / Open        |
| <kbd>Space</kbd> | Toggle multiselect    |
| <kbd>1..9</kbd>  | Jump to index in list |
