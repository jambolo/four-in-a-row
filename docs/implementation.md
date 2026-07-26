# Four In A Row — Game-Theoretic Facts and Implementation Notes

This document covers implementation conventions, the formal specification, AI/solver
facts, and strategy terminology for this project. The prose rules live in
[rules.md](rules.md).

> **Naming note:** this project is intentionally called "Four In A Row," not
> the trademarked name of the classic game it resembles. Do not introduce
> that trademarked name anywhere in code, comments, docs, or UI text.

## 1. Coordinates: implementation convention (0-indexed)

- Columns `0..=6` left to right; rows `0..=5` bottom to top.
- Cell index `(col, row)`; `row 0` is the bottom.
- Bottom-to-top row order is deliberate: gravity then means "smallest free row index",
  and win-direction vectors stay sign-symmetric.

## 2. Formal specification

The following is a complete, unambiguous restatement of the base game (see
[rules.md](rules.md) for the prose rules). `COLS = 7`, `ROWS = 6`, players are
`P1` and `P2`.

### 2.1 State

```text
State := {
  board:  Cell[COLS][ROWS]          // Cell ∈ { Empty, P1, P2 }; row 0 = bottom
  to_move: P1 | P2
  status:  InProgress | Win(P1) | Win(P2) | Draw
}

InitialState := {
  board:   all Empty
  to_move: P1
  status:  InProgress
}
```

### 2.2 Legal moves

```text
height(board, c)      := number of non-Empty cells in column c        // 0..ROWS
legal(board, c)       := 0 <= c < COLS  AND  height(board, c) < ROWS
legal_moves(state)    := { c : legal(state.board, c) }                // empty iff board full
```

### 2.3 Applying a move

```text
apply(state, c):
  require state.status == InProgress
  require legal(state.board, c)

  r := height(state.board, c)
  state.board[c][r] := state.to_move

  if wins(state.board, c, r, state.to_move):
      state.status := Win(state.to_move)
  else if |legal_moves(state)| == 0:
      state.status := Draw
  else:
      state.to_move := opponent(state.to_move)

  return state
```

### 2.4 Win detection

```text
DIRECTIONS := [ (1,0), (0,1), (1,1), (1,-1) ]

wins(board, c, r, player):
  for (dc, dr) in DIRECTIONS:
      n := 1
      n += run(board, c, r,  dc,  dr, player)     // forward from the new disc
      n += run(board, c, r, -dc, -dr, player)     // backward from the new disc
      if n >= 4: return true
  return false

run(board, c, r, dc, dr, player):
  k := 0
  loop:
      c += dc; r += dr
      if c,r outside board or board[c][r] != player: return k
      k += 1
```

### 2.5 Invariants

An implementation may assert all of these at every state:

- `0 <= count(P1) - count(P2) <= 1` — Player 1 has played the same number of discs as Player 2, or exactly one more.
- `count(P1) <= 21` and `count(P2) <= 21`.
- `to_move == P1` iff `count(P1) == count(P2)`.
- No column contains an `Empty` cell below a non-`Empty` cell (gravity / no floating discs).
- If `status != InProgress`, no further move is legal.
- At most one player has a four-in-a-row line in any reachable state.
- Total moves played `== count(P1) + count(P2)`, and is `<= 42`.

## 3. Game-theoretic facts

These are established results, useful for AI work and for testing a solver.

- This game is a **solved game**. It was solved independently by James Dow Allen (1 October 1988) and Victor Allis (16 October 1988). Allis used a knowledge-based approach built on nine strategic rules; modern solvers use minimax/negamax with alpha-beta pruning, move ordering, and transposition tables.
- There are **4,531,985,219,092** legal positions on the 7×6 board across 0–42 discs.
- **With perfect play the first player wins**, on or before move 41, by opening in the **center column (column 4)**.
- Opening move outcomes with perfect play by both sides:

| Opening column | Result for Player 1 | Move number |
| --- | --- | --- |
| 1 | Loss | 40 |
| 2 | Loss | 42 |
| 3 | Draw | — |
| 4 (center) | **Win** | ≤ 41 |
| 5 | Draw | — |
| 6 | Loss | 42 |
| 7 | Loss | 40 |

- The board is left-right symmetric, so column `c` and column `8 - c` are equivalent (1-indexed), or `c` and `6 - c` (0-indexed). Solvers exploit this to halve the root move list on the empty board.

## 4. Strategy concepts

Terminology used by the solver literature and useful for naming heuristics in code.

- **Threat** — an empty cell that would complete a four-in-a-row for a given player if that player occupied it.
- **Playable threat** — a threat sitting directly on top of the current stack in its column; it can be taken immediately.
- **Double threat / fork** — two threats a single move creates, at least one of which the opponent cannot answer. Usually decisive.
- **Trap** — placing a disc directly beneath an opponent's threat cell, handing them the winning square. A player must avoid being forced to fill the cell below an opponent's threat.
- **Zugzwang** — a position where a player would prefer to pass but must move, and every legal move worsens their position. In Four In A Row, control of zugzwang decides who is eventually forced to play beneath a critical cell.
- **Odd threat** — a threat on an odd-numbered row (rows 1, 3, 5, counting from the bottom).
- **Even threat** — a threat on an even-numbered row (rows 2, 4, 6).

Parity rules of thumb on the standard 7×6 board, given both players otherwise fill columns evenly:

- **Player 1 (first to move) benefits from odd threats**; **Player 2 benefits from even threats**. This follows from move parity: with an even number of empty cells remaining, it is Player 1's turn.
- Player 1 tends to win with one more unshared odd threat than Player 2 (and Player 2 holding no even threat).
- Player 2 tends to win with an even threat when Player 1 has none, or with two more unshared odd threats than Player 1.
- Where an odd and an even threat sit in the same column, the lower one dominates.
- Only odd-row critical cells shift the parity of the remaining empty cells, which is why odd/even threats are asymmetric in value.

Practical play principles:

- **Take the center.** Column 4 participates in more of the 69 winning lines than any other column; a common cell-weight heuristic is `[3, 4, 5, 7, 5, 4, 3]` per column.
- **Block immediate threats**, but check first whether you have your own winning move — a win beats a block.
- **Avoid moves that set up the opponent's threat cell** directly above your disc.
- **Build multiple threats**, not one; a single blockable threat achieves nothing.

## 5. Implementation notes for this project

- The rules are UI-independent and belong in the [src-core/](../src-core/) crate, per the crate-level note in [src-core/src/lib.rs](../src-core/src/lib.rs#L1-L10). The Tauri shell and the frontend must not re-implement move legality or win detection.
- Use the 0-indexed convention of §1 internally; convert only at the UI boundary.
- Because gravity forces the row, the move type should be a column index, not a cell.
- A 7×6 board fits a bitboard: 42 cells → two `u64` masks with a one-bit gap per column (7 columns × 7 bits = 49 bits) allows branch-free win detection via shifts of 1, 6, 7, and 8.
- The invariants in §2.5 make good property-test assertions; §3's opening-column table makes good integration tests for a solver.

## Sources

- [A Knowledge-based Approach of Connect-Four: The Game is Solved: White Wins — Victor Allis (PDF)](https://tromp.github.io/c4/connect4_thesis.pdf)
- [Expert Play in Connect-Four — John Tromp](https://tromp.github.io/c4.html)
- [Connect 4 Solver — Pascal Pons](https://connect4.gamesolver.org/)
- [Connect Four (MIT ES.268 course notes, PDF)](https://web.mit.edu/sp.268/www/2010/connectFourSlides.pdf)
