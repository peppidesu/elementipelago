# Elementipelago

A game inspired by the likes of doodle-god, infinitecraft and many other alchemy games like them.
Unlike the others, this game doesn't (yet) support sensible recipes but is instead built as an [Archipelago](archipelago.gg) first game.

## The randomisation

In this game a couple of things are randomised at the moment, using the terms as defined by archipelago we have:

Items:
- Elements: things you can use to merge with
- TODO: a filler item for which we still need to think of an alternative

Locations:
- Compounds: Things you have to make using the elements to send checks

## How to play

1. generate a multiworld with a slot using elementipelago as the game.
2. host it somewhere where https is setup (or localhost)
3. go to https://elementipelago.peppidesu.dev/ and connect to your slot.

## Options

As a player you can set how many elements you want to exists, how many intermediates, how many extra compounds (over the amount of items that always exist) should exist
and whether or not the compounds can be used in recipies themselves.

## What is being worked on still

- Polish
- Testing and possibly fixing what happens during a disconnect
- Useful items like "don't combine if you have the result" or "hide substances without a valid recipe right now" (subject to change)
- Giving items more fun names (want to relate to the location they were found/what item was found by them)
- Automatically picking an icon depending on the name
- Tracker integration (possibly behind an item)
