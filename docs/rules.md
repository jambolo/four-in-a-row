# Four In A Row — Complete Rules

This document is the authoritative rules reference for the game Four In A Row.

## 1. Overview

| Property | Value |
| --- | --- |
| Players | Exactly 2 |
| Type | Sequential, perfect information, zero-sum, deterministic |
| Board | 7 columns × 6 rows, vertical grid |
| Pieces | 21 discs per player (42 total) |
| Objective | Be first to form a line of four own discs |
| Possible outcomes | Player 1 wins, Player 2 wins, or draw |
| Hidden information | None |
| Randomness | None |

Two players alternate dropping one disc at a time into a vertically suspended grid.
A dropped disc falls to the lowest unoccupied cell of the chosen column. The first
player to get four of their own discs in an unbroken line — horizontal, vertical, or
diagonal — wins immediately. If all 42 cells fill with no such line, the game is a
draw.

## 2. Equipment

- A vertical grid with 7 columns and 6 rows (42 cells), open at the top of each column.
- 21 discs of color A (conventionally red) and 21 discs of color B (conventionally yellow).
- Physical sets also include a slider bar / release catch at the base used to empty the grid between games. This has no in-game rules effect.

Total discs (42) exactly equals total cells (42). Neither player can run out of
discs before the board is full: each player owns 21 discs, and no legal position can
require more than 21 of one color.

## 3. Coordinates and notation

Two conventions are used in this document. Both refer to the same board.

### 3.1 Human-facing convention (1-indexed)

- **Columns** are numbered `1`–`7`, left to right, from the players' viewpoint.
- **Rows** are numbered `1`–`6`, **bottom to top**. Row 1 is the floor of the grid.
- A cell is written `(column, row)`, e.g. `(4, 1)` is the bottom-center cell.
- Column 4 is the center column.

```text
row 6 |  .  .  .  .  .  .  .
row 5 |  .  .  .  .  .  .  .
row 4 |  .  .  .  .  .  .  .
row 3 |  .  .  .  .  .  .  .
row 2 |  .  .  .  .  .  .  .
row 1 |  .  .  .  .  .  .  .
       ---------------------
col      1  2  3  4  5  6  7
```

### 3.2 Move notation

A move is fully described by its **column alone** — the destination row is forced by
gravity. A game record is therefore a string of column digits in move order, e.g.
`4433` means: Player 1 → column 4, Player 2 → column 4, Player 1 → column 3,
Player 2 → column 3.

Standard move records use the 1-indexed columns (`1`–`7`), matching the published
solver literature.

## 4. Setup

1. The grid starts empty: all 42 cells unoccupied.
2. Each player takes all 21 discs of one color. Colors are chosen freely or by agreement.
3. Choose who moves first (see below). The player who moves first is **Player 1**; the other is **Player 2**.

**Deciding the first move:**

- For a single game: any mutually agreed method (choice, coin toss, youngest player, etc.).
- For a series of games, the published rule is that the players **alternate** who starts: whoever moved second in the previous game moves first in the next.
- Moving first is a genuine advantage (see §9), so competitive play must alternate or otherwise compensate.

## 5. Turn order

- Players alternate strictly: Player 1, Player 2, Player 1, …
- Player 1 makes moves 1, 3, 5, … (odd move numbers) and plays at most 21 discs.
- Player 2 makes moves 2, 4, 6, … (even move numbers) and plays at most 21 discs.
- **Passing is not allowed.** A player must move if any legal move exists.
- There is no move that removes or repositions a disc already on the board (see §10 for variants that change this).

## 6. Making a move

A turn consists of exactly one action: **drop one disc of your own color into one column.**

1. The player selects a column that is **not full** (i.e. it contains fewer than 6 discs).
2. The disc falls to the **lowest unoccupied cell** of that column.
3. The disc stays there permanently for the rest of the game.
4. The turn passes to the opponent, unless the move ended the game (§7, §8).

### 6.1 Legality

A move naming column `c` is legal if and only if:

- `c` is a valid column (1–7, or 0–6 zero-indexed), **and**
- column `c` currently contains fewer than 6 discs.

Dropping into a full column is **illegal**, not a pass and not a loss — the move is
simply rejected and the same player must choose another column. A digital
implementation should reject such input rather than change the turn.

At least one legal move exists whenever the board is not full, so a player is never
stuck without a move while the game is still in progress.

### 6.2 Determinism

Given a board state and a chosen column, the resulting position is uniquely
determined. There is no randomness, no simultaneous action, and no hidden state.

## 7. Winning

A player wins the instant the board contains **four or more of that player's discs in
an unbroken, consecutive line** along any of four directions:

| Direction | Description | Vector `(Δcol, Δrow)` |
| --- | --- | --- |
| Horizontal | Along a row | `(1, 0)` |
| Vertical | Up a column | `(0, 1)` |
| Diagonal ↗ | Up-right | `(1, 1)` |
| Diagonal ↘ | Down-right | `(1, -1)` |

Rules and clarifications:

- The four cells must be **adjacent** with no gap and no opposing disc between them.
- A line of **five or six** in a row also wins; it contains a line of four. Nothing requires the line to be exactly four.
- The game ends **immediately** when such a line appears. The winning player does not continue, and the opponent does not get a reply move.
- Only the player who just moved can have created a new line, since only their disc changed the board. A correct implementation only needs to check lines through the last-placed disc.
- It is **impossible for both players to complete a line on the same move**, because a move places exactly one disc of one color.

There are exactly **69 distinct four-in-a-row lines** on a 7×6 board:

| Direction | Count | Derivation |
| --- | --- | --- |
| Horizontal | 24 | 6 rows × 4 starting columns |
| Vertical | 21 | 7 columns × 3 starting rows |
| Diagonal ↗ | 12 | 4 starting columns × 3 starting rows |
| Diagonal ↘ | 12 | 4 starting columns × 3 starting rows |
| **Total** | **69** | |

## 8. Draws

The game is a **draw** if the board becomes completely full (all 42 cells occupied,
42 discs played) and neither player has a line of four.

- A draw is the only non-win terminal state in the base game.
- Because the board holds exactly 42 discs and both players have 21 each, a drawn game always ends after exactly 42 moves, with Player 2 making the final move (move 42).
- There is no repetition, stalemate, or move-limit rule — the board strictly fills, so every game terminates in at most 42 moves.
- Resignation and agreed draws are conventions of competitive play, not part of the published rules.

## 9. Game end summary

A game is over exactly when one of these holds, evaluated after each move:

1. The player who just moved has four or more in a row → that player **wins**; the opponent **loses**.
2. The board is full and condition 1 does not hold → **draw**.

Otherwise the game continues and the turn passes.

## 10. Official variants

These are published Hasbro variants. They are **not** part of the base game and
should be implemented, if at all, behind an explicit mode selection.

### 10.1 PopOut

- The board starts empty as normal.
- On a turn, a player either drops a disc in from the top **or** "pops out" one of their **own** discs from the **bottom row**, removing it from the board.
- Popping a disc out drops every disc above it in that column down one space.
- Win condition is unchanged: four in a row, horizontal, vertical, or diagonal.
- Because pops can create a line for *either* player simultaneously, a mode-specific rule is needed for the case where a pop completes lines for both colors; the common convention is that the player who made the move wins.
- Games are no longer bounded by 42 moves; a repetition or move-limit rule is required to guarantee termination.

### 10.2 Pop 10

- Setup: players alternately place their own discs to fill the grid from the bottom row upwards until all 42 cells are occupied.
- On a turn, a player removes one of their own discs through the bottom of the grid.
- If the removed disc was part of a four-in-a-row, the player sets it aside out of play and **immediately takes another turn**.
- If it was not part of a four-in-a-row, the disc is reinserted through a slot at the top into any open space.
- The first player to set aside **ten** discs wins.

### 10.3 Five-in-a-Row

- Played on a 6-row × 9-column grid.
- The outer columns are pre-filled with alternating discs before play.
- Five in a row is required to win.

### 10.4 Power Up

- Uses special marked discs (anvil, wall, bomb, ×2) with piece-specific effects such as popping an opponent's disc, blocking a cell, or granting a bonus turn.
- Otherwise follows the base game.

## Sources

- [Connect Four — Wikipedia](https://en.wikipedia.org/wiki/Connect_Four)
- [Official Rules and Instructions For Connect 4 Game — Hasbro](https://instructions.hasbro.com/en-in/instruction/connect-4-game)
- [Connect 4 Game Rules Manual — Hasbro](https://instructions.hasbro.com/en-ca/instruction/connect-4-game-instructions)
- [Connect Four — Wikibooks](https://en.wikibooks.org/wiki/Connect_Four)
