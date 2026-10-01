# 0003 The run

**Status:** implemented
**Date:** 2026-09-21

## Goal

A reason to hurry, and a cost for falling that stings without ending the
attempt.

## Behavior

**The clock** starts when the marble first moves, not when the course loads, so
a player can look around first. It counts up, in seconds and hundredths, and
stops when the goal is reached.

**Falling** past the course's floor level costs three seconds and puts the
marble back at the last checkpoint, still, with the clock running. A fall is
expensive but never fatal: there are no lives and no restart.

**Finishing** shows the time, the gems collected out of the total, and the
falls. That screen waits for Enter, which starts the course again from the
beginning with a fresh clock.

Those three lines sit in the middle of the window, evenly apart. One text with
newlines in it gave neither: the lines came out a font size apart, which is no
space between them at all, and the blank line before the prompt made that gap
twice the first one. A line each fixes it, and the gap is one number rather
than whatever the text layout does with a newline.

The middle of the window is the window's own width, which the engine says at
startup and on every resize. It was 400, which is the middle of a window 800
wide and nowhere near it in any other.

**Pausing** happens when the window loses focus, as in the other games on this
engine. Nothing moves and the clock stops.

**The display while running** is the clock, the gem count, and the falls, in the
top left, drawn as 2D text over the world.

## Acceptance criteria

- The clock does not run until the marble moves. — `run::tests::the_clock_waits_for_the_first_move`
- The clock runs while playing. — `run::tests::the_clock_runs`
- The clock stops at the goal. — `run::tests::the_goal_stops_the_clock`
- A fall costs three seconds. — `run::tests::a_fall_costs_time`
- A fall returns the marble to the last checkpoint, at rest. — `run::tests::a_fall_returns_to_the_checkpoint`
- Falls are counted. — `run::tests::falls_are_counted`
- Finishing keeps the time, the gems and the falls. — `run::tests::finishing_keeps_the_result`
- Starting again clears everything. — `run::tests::a_new_run_starts_clean`
- The clock stops while paused. — `run::tests::pausing_stops_the_clock`
- The finish readout sits in the middle of the window, whatever its width. — `marble_game::tests::the_finish_readout_sits_in_the_middle_of_the_window`
- And its lines are evenly apart. — `marble_game::tests::the_finish_readout_is_evenly_spaced`

### Verified by hand

- The three second penalty is felt but not ruinous, and a fall never leaves the
  marble somewhere it cannot recover from.

## Out of scope

Saved best times, a leaderboard, medals, lives, and any difficulty setting.
