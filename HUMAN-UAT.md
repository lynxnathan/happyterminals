# Cross-Terminal Verification Matrix (M1 Exit -- BACK-05)

**Date:** ____
**Tester:** ____
**Commit:** ____

## Test Command

```bash
cargo run -p happyterminals --example spinning-cube
```

## Verification Criteria

For each terminal, verify ALL of the following:

1. Cube renders visibly (ASCII characters forming a 3D cube shape)
2. Cube rotates smoothly at ~30fps (no visible stuttering)
3. Coalesce startup effect plays (characters reform from dissolved state)
4. No visible flicker or tearing
5. Ctrl-C exits cleanly (cursor visible, prompt responsive, no garbage characters)
6. Terminal state is sane after exit (can type commands normally)

## Matrix

| # | Terminal | OS | Status | Notes |
|---|---------|-----|--------|-------|
| 1 | Windows Terminal | Windows | [ ] Pass / [ ] Fail / [ ] N/A | |
| 2 | GNOME Terminal | Linux | [ ] Pass / [ ] Fail / [ ] N/A | |
| 3 | Terminal.app | macOS | [ ] Pass / [ ] Fail / [ ] N/A | |
| 4 | iTerm2 | macOS | [ ] Pass / [ ] Fail / [ ] N/A | |
| 5 | Kitty | Linux/macOS | [ ] Pass / [ ] Fail / [ ] N/A | |
| 6 | Alacritty | Cross-platform | [ ] Pass / [ ] Fail / [ ] N/A | |
| 7 | tmux (inside any terminal) | Any | [ ] Pass / [ ] Fail / [ ] N/A | |
| 8 | screen (inside any terminal) | Any | [ ] Pass / [ ] Fail / [ ] N/A | |
| 9 | SSH session (from any client) | Remote Linux | [ ] Pass / [ ] Fail / [ ] N/A | |

## Additional Notes

(Record any terminal-specific quirks, color rendering differences, etc.)

## Sign-off

- [ ] All accessible terminals pass
- [ ] Untested terminals documented with reason
- [ ] BACK-05 satisfied
