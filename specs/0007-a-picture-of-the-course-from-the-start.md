# 0007 A picture of the course from the start

**Status:** implemented
**Date:** 2026-11-03

## Goal

The frame marble puts on its readme is held against a picture of what that
frame should be, so a change to the course, the camera or the skin is seen
rather than shipped.

## Why it belongs here

Spec 0002 says where the course is, 0006 says how the marble rolls on it,
and every test of either is arithmetic: a position, a speed, a fall. None of
them can see the course.

The engine's spec 0076 made the rest of it one call, so what a game writes is
the reference, the size and what to draw.

**And nothing is staged, because the opening frame is the photograph.**
marble is the only one of these games that photographs what it opens on: the
marble on the first slab, the course ahead of it, the gems along it and the
two ramps. `--screenshot` here only keeps the game from pausing when it
loses focus, since a staged run is photographed from behind the terminal.
A test never loses focus, so it never reads that.

## Behavior

**One `update`, at no time at all.** The camera is worked out from where the
marble is rather than left where `Follow::new` put it, and a `dt` of nought
means nothing rolls between that and the shutter.

**The picture is the 3D.** The engine's harness leaves the overlay empty, so
the clock and the gem count are not in shot and a reference does not change
when a line of help does.

**It is the engine's size**, 320 by 240, said once in `picture::SIZE` and
read here.

## Acceptance criteria

- The course from the start matches its
  picture. — `marble_game::tests::the_course_from_the_start_looks_right`

### Verified by hand

- **The picture itself.** The marble on the first slab, the blue section and
  its two ramps ahead, the green shelf to the left, the gems above: the same
  framing the committed `media/screenshot.png` shows, without the window or
  the readout.
- **A control.** The marble started six tenths of a unit to one side and the
  test failed on 10.9% of its pixels, which is large because the camera
  follows the marble and the whole frame moves with it. Put back and
  compared.

## Out of scope

**The readout.** It is 2D and the engine's harness draws none of it. The
clock has its own test in this file.

**A picture of a later part of the course.** Reaching one means rolling the
marble there, which is spec 0006's arithmetic rather than a photograph, and
a reference per slab is a flipbook nobody reads.
