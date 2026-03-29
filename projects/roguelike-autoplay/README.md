# Roguelike Autoplay (Sigil Project)

Tiny seeded ASCII roguelike demo with branching dungeon generation, enemy roles, ranged combat, consumables, treasure scoring, and an animated auto-play replay.

Glyphs:
- `@` player
- `g` goblin
- `a` archer
- `B` brute
- `s` shaman
- `!` potion
- `*` bomb
- `?` blink
- `/` spear
- `$` treasure
- `>` exit

The player starts with a sword, bow, arrows, and one potion. The auto-player can swap to a spear pickup, shoot at range, throw bombs, blink out of danger, and still plays as a deterministic replay under seeded randomness.

Run from repo root:

```bash
cargo run -q -p sigil-cli --manifest-path language/compiler/Cargo.toml -- run projects/roguelike-autoplay/src/main.sigil
```
