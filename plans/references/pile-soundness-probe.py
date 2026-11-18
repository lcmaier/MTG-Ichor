"""Is a pile never wrong, and how often is it exact?

    python plans/references/pile-soundness-probe.py            # 300 games of 6 cards, about a minute
    python plans/references/pile-soundness-probe.py 100 7      # games, cards

Behind `information-model-design`'s decision 1, option F
(`plans/handoffs/information-model-design.md`). A *pile* is a group of hidden
cards a player cannot tell apart by place: the player knows which faces are
in the pile and not which is where. This plays small random games of one
seat's hidden cards (a library in order, a hand, face-down objects) watched
by an opponent, and keeps two things side by side:

- **the truth of what the watcher can know**: every arrangement of the cards
  still consistent with everything the watcher was shown, by brute force;
- **the piles**, kept by five rules: a card shown is a pile of one; places
  scrambled out of sight join one pile; a card moved from a pile no one can
  point into takes its pile with it; a hand is one pile, since a hand has no
  order; a card that leaves hiding leaves its pile.

After every event it asks two questions. Sound: is every arrangement the
truth allows also allowed by the piles? If not, the piles would show the
watcher something false. Exact: do the piles allow nothing more?

The watcher is taken to know which cards the deck holds, so "every
arrangement" is finite. That is the hardest case for exactness, and it is
what a player knows of their own deck.
"""
import itertools
import random
import sys


def canon(lib, hand, down):
    return (tuple(lib), frozenset(hand), tuple(down))


def step_truth(worlds, event, arg, seen):
    """The arrangements consistent with what the watcher has been shown."""
    out = set()
    for lib, hand, down in worlds:
        if event == "draw":
            out.add(canon(lib[1:], hand | {lib[0]}, down))
        elif event == "play":            # a hand card played face up: its face is seen
            if arg in hand:
                out.add(canon(lib, hand - {arg}, down))
        elif event == "reveal_top":
            if lib[0] == seen:
                out.add((lib, hand, down))
        elif event == "reveal_hand":
            if hand == seen:
                out.add((lib, hand, down))
        elif event == "shuffle":
            for order in itertools.permutations(lib):
                out.add(canon(order, hand, down))
        elif event == "put_back":        # a hand card onto the library, unseen
            for card in hand:
                out.add(canon((card,) + lib, hand - {card}, down))
        elif event == "bottom":          # the top `arg` cards to the bottom, in an order unseen
            for order in itertools.permutations(lib[:arg]):
                out.add(canon(lib[arg:] + order, hand, down))
        elif event == "reorder_top":     # the top `arg` cards back in an order unseen
            for order in itertools.permutations(lib[:arg]):
                out.add(canon(order + lib[arg:], hand, down))
        elif event == "cast_face_down":  # a hand card becomes a face-down object, unseen
            for card in hand:
                out.add(canon(lib, hand - {card}, down + (card,)))
        elif event == "turn_up":         # a face-down object is shown and leaves
            if down[arg] == seen:
                out.add(canon(lib, hand, down[:arg] + down[arg + 1:]))
    return out


class Piles:
    """What the engine would keep: each hidden card's pile, by the five rules."""

    def __init__(self, cards, one_pile_a_player=False):
        self.pile = {card: 0 for card in cards}   # one pile: the watcher knows the deck, not its order
        self.next = 1
        # Option E's grain, for comparison: whatever the watcher cannot point
        # at is one pile, "seen, still hidden", however it got there.
        self.one_pile_a_player = one_pile_a_player

    def alone(self, card):
        self.pile[card] = self.next
        self.next += 1

    def join(self, cards):
        cards = list(cards)
        if not cards:
            return
        if self.one_pile_a_player:
            cards += [card for card in self.pile if not self.known_alone(card)]
        touched = {self.pile[card] for card in cards}
        target = min(touched)
        for card, pile in self.pile.items():
            if pile in touched:
                self.pile[card] = target

    def leave(self, card):
        del self.pile[card]

    def known_alone(self, card):
        return sum(1 for pile in self.pile.values() if pile == self.pile[card]) == 1

    def allowed(self, lib, hand, down):
        """Every arrangement the piles allow: each pile's cards, in any order, over the places they hold."""
        place = {}
        for at, card in enumerate(lib):
            place[card] = ("lib", at)
        for card in hand:
            place[card] = ("hand", None)
        for at, card in enumerate(down):
            place[card] = ("down", at)
        groups = {}
        for card, pile in self.pile.items():
            groups.setdefault(pile, []).append(card)
        worlds = set()
        for choice in itertools.product(*(itertools.permutations(group) for group in groups.values())):
            new_lib, new_hand, new_down = list(lib), set(), list(down)
            for group, order in zip(groups.values(), choice):
                for holder, card in zip(group, order):
                    zone, at = place[holder]
                    if zone == "lib":
                        new_lib[at] = card
                    elif zone == "hand":
                        new_hand.add(card)
                    else:
                        new_down[at] = card
            worlds.add(canon(new_lib, new_hand, new_down))
        return worlds


def play_one(rng, cards, one_pile_a_player):
    lib = tuple(rng.sample(cards, len(cards)))
    hand, down = frozenset(), ()
    truth = {canon(order, hand, down) for order in itertools.permutations(cards)}
    piles = Piles(cards, one_pile_a_player)
    sound = exact = steps = 0
    excess = 0.0
    for _ in range(14):
        options = ["shuffle"] if len(lib) > 1 else []
        if lib:
            options += ["draw", "reveal_top"]
        if len(lib) >= 2:
            options += ["bottom", "reorder_top"]
        if hand:
            options += ["play", "reveal_hand", "put_back", "cast_face_down"]
        if down:
            options += ["turn_up"]
        if not options:
            break
        event = rng.choice(options)
        arg = seen = None
        if event == "draw":
            card = lib[0]
            if not piles.known_alone(card):                  # the watcher cannot say which card arrived
                piles.join([card] + [c for c in hand if not piles.known_alone(c)])
            lib, hand = lib[1:], hand | {card}
        elif event == "play":
            arg = rng.choice(sorted(hand))
            hand = hand - {arg}
            piles.leave(arg)
        elif event == "reveal_top":
            seen = lib[0]
            piles.alone(seen)
        elif event == "reveal_hand":
            seen = hand
            for card in hand:
                piles.alone(card)
        elif event == "shuffle":
            piles.join(lib)
            lib = tuple(rng.sample(lib, len(lib)))
        elif event == "put_back":
            card = rng.choice(sorted(hand))
            piles.join(hand)                                 # any of them could be the one that went
            lib, hand = (card,) + lib, hand - {card}
        elif event == "bottom":
            arg = rng.randint(2, len(lib))
            moved = list(lib[:arg])
            piles.join(moved)
            rng.shuffle(moved)
            lib = lib[arg:] + tuple(moved)
        elif event == "reorder_top":
            arg = rng.randint(2, len(lib))
            moved = list(lib[:arg])
            piles.join(moved)
            rng.shuffle(moved)
            lib = tuple(moved) + lib[arg:]
        elif event == "cast_face_down":
            card = rng.choice(sorted(hand))
            piles.join(hand)
            hand, down = hand - {card}, down + (card,)
        elif event == "turn_up":
            arg = rng.randrange(len(down))
            seen = down[arg]
            down = down[:arg] + down[arg + 1:]
            piles.leave(seen)
        truth = step_truth(truth, event, arg, seen)
        assert canon(lib, hand, down) in truth, "the brute force lost the real game"
        allowed = piles.allowed(lib, hand, down)
        steps += 1
        sound += truth <= allowed
        exact += truth == allowed
        excess += len(allowed) / len(truth)
    return steps, sound, exact, excess


def main(argv):
    games = int(argv[0]) if argv else 300
    size = int(argv[1]) if len(argv) > 1 else 6
    for name, one_pile_a_player in (("piles, by the five rules (option F)", False),
                                    ("one pile a player: seen, still hidden (option E)", True)):
        rng = random.Random(20261117)    # the same games for both
        steps = sound = exact = 0
        excess = 0.0
        for _ in range(games):
            s, so, ex, more = play_one(rng, list(range(size)), one_pile_a_player)
            steps, sound, exact, excess = steps + s, sound + so, exact + ex, excess + more
        print(f"{name}: {games} games of {size} cards, {steps} events")
        print(f"  sound (allows every arrangement the truth does): {sound} of {steps}")
        print(f"  exact (and allows nothing more):                 {exact} of {steps}, {100 * exact / steps:.1f}%")
        print(f"  arrangements allowed for each the truth allows:  {excess / steps:.2f} on average")


if __name__ == "__main__":
    main(sys.argv[1:])
