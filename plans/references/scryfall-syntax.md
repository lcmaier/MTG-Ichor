# Scryfall Search Syntax Reference

Source: https://scryfall.com/docs/syntax (fetched 2026-08-20)

Filter syntax for `https://api.scryfall.com/cards/search?q=...`, useful for finding
example cards during rules-corpus research (e.g. "which cards grant the basic
supertype"). This is a tooling reference, not a rules authority — the Comprehensive
Rules (`MTG-Rules/`) remain the sole source of truth for engine behavior, per
`CLAUDE.md`. Query the API from Bash with a descriptive `User-Agent` header;
`WebFetch` gets 403 from Scryfall's origin.

## Colors and Color Identity

You can find cards that are a certain color using the c: or color: keyword,
and cards that are a certain color identity using the id: or identity: keywords.
Both sets of keywords accepts full color names like blue or the abbreviated color letters w, u, r, b and g.
You can use many nicknames for color sets:
all guild names (e.g. azorius), all shard names (e.g. bant),
all college names (e.g., quandrix),
all wedge names (e.g. abzan),
and the four-color nicknames chaos, aggression, altruism, growth, artifice are supported.
Use c or colorless to match colorless cards, and m or multicolor to match multicolor cards.
You can use comparison expressions ( >, <, >=, <=, and !=)
to check against ranges of colors. You can also use numbers instead to find cards that have that many colors.
Find cards that have a color indicator with has:indicator.

Examples:

- `c:rg` — Cards that are red and green
- `color>=uw -c:red` — Cards that are at least white and blue, but not red
- `id<=esper t:instant` — Instants you can play with an Esper commander
- `id:c t:land` — Land cards with colorless identity
- `c=2 is:bear` — 'Bears' that are exactly two colors.

## Card Types

Find cards of a certain card type with the t: or type: keywords.
You can search for any supertype, card type, or subtype.
Using only partial words is allowed.

Examples:

- `t:merfolk t:legend` — Legendary merfolk cards
- `t:goblin -t:creature` — Goblin cards that aren’t creatures

## Card Text

Use the o: or oracle: keywords to find cards that have specific
phrases in their text box.
You can put quotes " " around text with punctuation or spaces.
You can use ~ in your text as a placeholder for the card’s name.
This keyword usually checks the current Oracle text for cards,
so it uses the most up-to-date phrasing available.
For example, “dies” instead of “is put into a graveyard”.
Use the fo: or fulloracle: operator to search the full Oracle text,
which includes reminder text.
You can also use keyword: or kw: to search for cards with a
specific keyword ability.

Examples:

- `o:draw t:creature` — Creatures that deal with drawing cards
- `o:"~ enters tapped"` — Cards that enter the battlefield tapped
- `kw:flying -t:creature` — Noncreatures that have the flying keyword

## Mana Costs

Use the m: or mana: keyword to search for cards that have
certain symbols in their mana costs.
This keyword uses the official text version of mana costs set
forth in the Comprehensive Rules.
For example, {G} represents a green mana.
Shorthand is allowed for symbols that aren’t split: G is the same as {G}
However, you must always wrap complex/split symbols like {2/G} in braces.
You can search for mana costs using comparison operators; a mana cost is greater than another if it includes
all the same symbols and more, and it’s less if it includes only a subset of symbols.
You can find cards of a specific mana value with manavalue or mv,
comparing with a numeric expression ( >, <, =, >=, <=, and !=).
You can also find even or odd mana costs with manavalue:even or manavalue:odd
You can filter cards that contain hybrid mana symbols with is:hybrid or Phyrexian mana symbols with is:phyrexian
You can find permanents that provide specific levels of devotion, using either single-color
mana symbols for devotion to one color, or hybrid symbols for devotion to two, with devotion: or a comparison operator.
You can also find cards that produce specific types of mana, with produces:

Examples:

- `mana:{G}{U}` — Cards with one green and blue mana in their costs
- `m:2WW` — Cards with two generic and two white mana in their cost
- `m>3WU` — Cards that cost more than three generic, one white, and one blue mana
- `m:{R/P}` — Cards with one Phyrexian red mana in their cost
- `c:u mv=5` — Blue cards with mana value 5
- `devotion:{u/b}{u/b}{u/b}` — Cards that contribute 3 to devotion to black and blue
- `produces=wu` — Cards that produce blue and white mana

## Power, Toughness, and Loyalty

You can use numeric expressions ( >, <, =, >=, <=, and !=)
to find cards with certain
power, power / pow, toughness, toughness / tou,
total power and toughness, pt / powtou,
or starting loyalty, loyalty / loy.
You can compare the values with each other or with a provided number.

Examples:

- `pow>=8` — Cards with 8 or more power
- `pow>tou c:w t:creature` — White creatures that are top-heavy
- `t:planeswalker loy=3` — Planeswalkers that start at 3 loyalty

## Multi-faced Cards

You can find cards that have more than one face with is:split (split cards), is:flip (flip cards), is:transform (cards that transform, alias tdfc), is:meld (cards that meld), is:leveler (cards with Level Up), is:dfc (double-faced cards), and is:mdfc (modal double-faced cards).
You can find meld parts specifically with is:meldpart and their results with is:meldresult.

Examples:

- `is:meld` — Cards that meld
- `is:split` — Split-faced cards

## Spells, Permanents, and Effects

Find cards that are cast as spells with is:spell.
Find permanent cards with is:permanent,
historic cards with is:historic,
creatures that can be in your party with is:party,
creatures that are outlaws with is:outlaw,
modal effects with is:modal,
vanilla creatures with is:vanilla, 
French vanilla cards with is:frenchvanilla,
2/2/2 “bear” creatures with is:bear,
or lands that can turn into creatures with is:manland.

Examples:

- `c>=br is:spell f:duel` — Black and red multicolor spells in Duel Commander
- `is:permanent t:rebel` — Rebel permanents
- `is:vanilla` — Vanilla creatures

## Extra Cards and Funny Cards

Vanguard, plane, scheme,
and phenomenon cards are hidden by default, as are
cards from “memorabilia” sets.
You must either search for their type
(using the type: keyword)
or a set that contains them (the set: keyword).
Un-cards, holiday cards, and
other funny cards are findable with is:funny or mentioning their set.
You may also use include:extras to
reveal absolutely every card when you search.

Examples:

- `is:funny` — All funny cards
- `t:scheme` — Scheme cards
- `power include:extras` — Cards with “power” in their name, including extras

## Rarity

Use r: or rarity: to find cards by their print rarity.
You can search for common, uncommon, rare, special, mythic, and bonus.
You can also use comparison operators like < and >=.
Use new:rarity to find reprint cards printed at a new
rarity for the first time. You can find cards that have
ever been printed in a given rarity using in: (for example, in:rare to find cards that have ever been printed at rare.)
Cards new to pauper in particular can be found using is:newinpauper.

Examples:

- `r:common t:artifact` — Common artifacts
- `r>=r` — Cards at rare rarity or above (i.e., rares and mythics)
- `rarity:common e:ima new:rarity` — Cards printed as commons for the first time in Iconic Masters
- `in:rare -rarity:rare` — Non-rare printings of cards that have been printed at rare

## Sets and Blocks

Use s:, e:, set:, or edition: to find cards
using their Magic set code.
Use cn: or number: to find cards by collector number within a set.
Combine this with s: to find specific card editions.
Searching by ranges with a syntax like cn>50 is also possible.
Use b: or block: to find cards in a Magic block by
providing the three-letter code for any set in that block.
Use g: or group: to find cards in sets directly tied to a particular set release 
by providing its three-letter code. For example, g:ecc returns cards in Lorwyn Eclipsed 
Commander’s parent set ( ecl), sibling sets ( tecl, aecl, etc), child sets ( tecc), and ecc itself.
The in: keyword finds cards that once “passed through”
the given set code. For example in:lea would only match cards
that once appeared in Alpha.
You can search for cards based on the type of product they appear in.
This includes the primary product types ( st:core, st:expansion,
or st:draftinnovation), as well as series of products
( st:masters, st:funny, st:commander, st:duel_deck, st:from_the_vault, st:spellbook, or st:premium_deck) and more
specialized types ( st:alchemy, st:archenemy, st:masterpiece, st:memorabilia, st:planechase, st:promo, st:starter, st:token, st:treasure_chest, or st:vanguard.)
The in: keyword also supports these set types, so you can search
for cards with no printings in a set type with a query like -in:core.
You can also search for individual cards that were sold in certain places
with is:booster or is:planeswalker_deck, or specific types of promo
cards with is: queries like is:league, is:buyabox, is:giftbox, is:intro_pack, is:gameday, is:prerelease, is:release, is:fnm, is:judge_gift, is:arena_league, is:player_rewards, is:media_insert, is:instore, is:convention, or is:set_promo, among others.

Examples:

- `e:war` — Cards from War of the Spark
- `e:war is:booster` — Cards available inside War of the Spark booster boxes
- `b:wwk` — Cards in Zendikar Block (but using the Worldwake code)
- `g:fin or g:fic` — Cards from sets related directly to Final Fantasy or its Commander deck
- `in:lea in:m15` — Cards that were in both Alpha and Magic 2015
- `t:legendary -in:booster` — Legendary cards that have never been printed in a booster set
- `is:datestamped is:prerelease` — Prerelease promos with a date stamp

## Cubes

Find cards that are part of cube lists using the cube: keyword. The currently supported cubes are arena, grixis, legacy, chuck, twisted, april, protour, uncommon, modern, amaz, tinkerer, livethedream, chromatic, vintage, and apcube.

Examples:

- `cube:vintage` — Cards in the Vintage Cube
- `cube:modern t:planeswalker` — Planeswalkers in the Modern Cube

## Format Legality

Use the f: or format: keywords
to find cards that are legal in a given format.
You can also find cards that are explicitly
banned in a format with the banned: keyword and
restricted with the restricted: keyword.
The current supported formats are: standard, future (Future Standard), historic, timeless, gladiator, pioneer, modern, legacy, pauper, vintage, penny (Penny Dreadful), commander, oathbreaker, standardbrawl, brawl, competitivebrawl, alchemy, paupercommander, duel (Duel Commander), oldschool (Old School 93/94), premodern, predh, and tlr (Tiny Leaders: Reborn).
You can use is:commander to find cards that can be your commander, is:brawler to find cards that can be your Brawl Commander, is:companion to find Companion cards, is:duelcommander to find cards that can be your Duel Commander, and is:oathbreaker to find cards that can be your Oathbreaker.
You can find cards with any flavor of Commander Partner mechanic with is:partner.
You can filter cards by their EDHREC ranking using edhrecrank or edhrec along with a numeric expression ( >, <, =, >=, <=, and !=).
These rankings start from 1 and increase with decreasing popularity, and excludes basic lands. 
If you want less popular cards than a particular rank, you must use > to seek cards with a higher numerical placement.
Cards that are Commander Gamechangers can be found using is:gamechanger.
You can find cards on the Reserved List with is:reserved.

Examples:

- `c:g t:creature f:pauper` — Green creatures in Pauper format
- `banned:legacy` — Cards banned in Legacy format
- `is:commander` — Cards that can be your commander
- `is:reserved` — Cards on the Reserved List
- `edhrecrank<=100 sort:edhrec` — The Top 100 cards by EDHREC Ranking (excluding basics) sorted by their ranking

## USD/EUR/TIX prices

You can find prints within certain usd, eur, tix price
ranges by comparing them with a
numeric expression ( >, <, =, >=, <=, and !=).
You can find the cheapest print of each card with cheapest:usd, cheapest:eur, and cheapest:tix.

Examples:

- `tix>15.00` — Cards that cost more than 15 TIX at MTGO stores
- `usd>=0.50 e:ema` — Cards worth 50¢ or more in Eternal Masters

## Artist, Flavor Text and Watermark

Search for cards illustrated by a
certain artist with the a:, or artist: keywords.
And you can search for cards with more than one artist using artists>1.
Search for words in a card’s flavor text using the ft: or flavor: keywords.
Search for a card’s affiliation watermark using the wm: or watermark: keywords, or match all cards with
watermarks using has:watermark.
For any of these, you can wrap statements with spaces
or punctuation in quotes " ".
You can find cards being printed with new illustrations using new:art, being illustrated by a particular artist for the
first time with new:artist, and with brand-new flavor text
using new:flavor.
You can compare how many different illustrations a give card
has with things like illustrations>1.

Examples:

- `a:"proce"` — Cards illustrated by Vincent Proce
- `ft:mishra` — Cards that mention Mishra in their flavor text
- `ft:designed e:m15` — Cards created by guest designers in Magic 2015
- `wm:orzhov` — Cards with Orzhov guild watermark
- `e:m10 new:art is:reprint` — Reprints with new art in Magic 2010
- `new:art -new:artist st:masters game:paper` — Cards in masters sets with new art by the same artist
- `new:flavor e:m15 is:reprint` — Reprint cards in Magic 2015 which have new flavor text

## Border, Frame, Foil & Resolution

Use the border: keyword
to find cards with a black, white, silver, or borderless border.
You can find cards with a specific frame edition using frame:1993, frame:1997, frame:2003, frame:2015, and frame:future.
You can also search for particular frame-effects, such as frame:legendary, frame:colorshifted, frame:tombstone, frame:enchantment.
You can find cards with full art using is:full.
new:frame will find cards printed in a specific frame
for the first time.
Each card is available in non-foil, in foil, or in both. You can find
prints available in each with is:nonfoil and is:foil, or is:foil is:nonfoil to find prints (like most booster cards)
available in both. You can also find cards available in etched foil
and glossy finishes with is:etched and is:glossy.
You can find cards in our database with high-resolution images
using is:hires.
Search for a card’s security stamp with stamp:oval, stamp:acorn, stamp:triangle, or stamp:arena
You can search for or exclude Universes Beyond cards with is:universesbeyond or not:universesbeyond. You can search for cards with the default Magic frame with is:default, 
or for atypical frame treatments with is:atypical.

Examples:

- `border:white t:creature` — White-bordered creature cards
- `is:new r:mythic` — Mythic cards with the 2015 holofoil-stamp frame
- `is:old t:artifact` — Artifacts in either the 1993 or 1997 variant of the 'classic' frame
- `is:hires` — Cards with high-resolution scans
- `is:foil e:c16` — Commander 2016 cards printed in foil
- `frame:2003 new:frame in:fut is:reprint` — Future cards printed later in other frames

## Games, Promos, & Spotlights

You can find specific prints available in different Magic game
environments with the game: keyword.
The games paper, mtgo, and arena are supported.
You can filter by a card’s availability in a game with the in: keyword.
The games paper, mtgo, and arena are supported.
Find prints that are only available digitally (MTGO and Arena) with is:digital.
You can find Arena Alchemy cards with is:alchemy, and Arena Rebalanced cards with is:rebalanced.
Find promotional cards (in any environment) with is:promo.
Find cards that are Story Spotlights with is:spotlight.
Find cards that Scryfall has had the honor of previewing with is:scryfallpreview.

Examples:

- `game:arena` — Cards available on MTG:Arena
- `-in:mtgo f:legacy` — Legacy legal cards not available on MTGO
- `is:promo` — Promotional cards
- `is:spotlight` — Story Spotlight cards

## Year

You can use numeric expressions ( >, <, =, >=, <=, and !=)
to find cards that were released relative to a certain year or a yyyy-mm-dd date.
You can also use any set code to stand in for the set’s release date, or use now / today to stand in for today’s date.

Examples:

- `year<=1994` — Cards from 1994 and before
- `year=2026` — Cards released this year
- `date>=2015-08-18` — Cards printed on or after August 18, 2015
- `date>ori` — Cards printed in sets released after Magic Origins
- `date>now` — Cards that have release dates in the future

## Tagger Tags

You can use art:, atag:, or arttag: to find things in a card’s illustration.
You can use function:, otag:, or oracletag: to find “Oracle” tags which
describe the function of the card.
Data for these two features comes from the Tagger project.

Examples:

- `art:squirrel` — Art that contains a squirrel
- `function:removal` — Cards that cause removal

## Reprints

You can find reprints with is:reprint, cards that
were new in their set with not:reprint, and cards that
have only been in a single set with is:unique. You can also compare
the number of times a card has been printed with syntax like prints=1,
or the number of sets a card has been in with sets=1. These can also
be compared including only paper sets with paperprints=1 and papersets=1.

Examples:

- `e:c16 not:reprint` — Cards that were new in Commander 2016
- `e:ktk is:unique` — Cards that were in Khans of Tarkir and not printed in any other set
- `sets>=20` — Cards that have been printed in 20 or more distinct sets
- `e:arn papersets=1` — Cards that were printed in Arabian Nights but never reprinted in paper

## Languages

You can request cards in certain languages with the lang: / language: keywords.
You can widen your search to any language with the special lang:any keyword.
You can also find the first printing of a card in each language using new:language and all printings of a card that’s been printed in a language at least once with in: (e.g. in:ru will find cards that have ever been printed in Russian.)

Examples:

- `lang:japanese` — Cards in Japanese
- `lang:any t:planeswalker unique:prints` — Planeswalkers in any language
- `lang:ko new:language t:goblin` — The first printings of goblin cards in the Korean language
- `in:ru in:zhs` — Cards that have been printed in both Russian and Simplified Chinese

## Shortcuts and Nicknames

The search system includes a few convenience shortcuts for
common card sets:
You can find some interesting land groups with is:bikeland (alias cycleland, bicycleland), is:bondland (alias crowdland, bbdland, battlebondland) is:bounceland (alias karoo), is:canopyland (alias canland), is:checkland, is:creatureland, is:dual, is:fastland, is:fetchland, is:filterland, is:gainland, is:painland, is:pathway, is:scryland, is:surveilland, is:shadowland (alias snarl), is:shockland, is:slowland, is:storageland, is:tangoland (alias battleland), is:tricycleland (alias trikeland, triome), is:triland,
and is:tangoland (alias battleland)
You can find all Masterpiece Series cards with is:masterpiece

Examples:

- `is:dual` — Cards that are dual lands
- `is:fetchland` — Cards that are fetchlands
- `is:colorshifted` — Colorshifted cards

## Negating Conditions

All keywords except for include can be negated by
prefixing them with a hyphen ( -).
This inverts the meaning of the keyword to reject
cards that matched what you’ve searched for.
The is: keyword has a convenient inverted mode not: which is the same as -is:.
Conversely, -not: is the same as is:.
Loose name words can also be inverted with -

Examples:

- `-fire c:r t:instant` — Red instants without the word “fire” in their name
- `o:changeling -t:creature` — Changeling cards that aren’t creatures
- `not:reprint e:c16` — Cards in Commander 2016 that aren’t reprints

## Regular Expressions

Use forward slashes // with the type:, t:, oracle:, o:, flavor:, ft:,
and name: keywords
to match those parts of a card with a regular expression.
Scryfall supports
many regex features such as.*?,
option groups (a|b),
brackets [ab],
character classes \d, \w,
and anchors (?!), \b, ^,
and $.
Forward slashes inside your regex must be escaped with \/.
Full documentation for this keyword is available on
our Regular Expressions help page.

Examples:

- `t:creature o:/^{T}:/` — Creatures that tap with no other payment
- `t:instant o:/\spp/` — Instants that provide +X/+X effects
- `name:/\bizzet\b/` — Card names with “izzet” but not words like “mizzet”

## Exact Names

If you prefix words or quoted phrases with ! you will find cards with that exact name only.
This is still case-insensitive.

Examples:

- `!fire` — The card Fire
- `!"sift through sands"` — The card Sift Through Sands

## Using “OR”

By default every search term you enter is combined.
All of them must match to find a card.
If you want to search over a set of options or choices,
you can put the special word or / OR between terms.

Examples:

- `t:fish or t:bird` — Cards that are Fish or Birds
- `t:land (a:titus or a:avon)` — Lands illustrated by Titus Lunter or John Avon

## Nesting Conditions

You may nest conditions inside parentheses () to group them together.
This is most useful when combined with the OR keyword.
Remember that terms that are not separated by OR are still combined.

Examples:

- `t:legendary (t:goblin or t:elf)` — Legendary goblins or elves
- `through (depths or sands or mists)` — The Unspeakable combo

## Display Keywords

You can enter your display options for searches as
keywords rather than using the controls on the page.
Select how duplicate results are eliminated with unique:cards, unique:prints (previously ++), or unique:art (also @@).
Change how results are shown with display:grid, display:checklist, display:full, or display:text.
Change how results are sorted with order:artist, order:cmc, order:power, order:toughness, order:set, order:name, order:usd, order:tix, order:eur, order:rarity, order:color, order:released, order:spoiled, order:edhrec, order:penny, order:review, or order:imageupdated.
Select what printings of cards to preferentially show with prefer:oldest, prefer:newest, prefer:usd-low or prefer:usd-high (and 
the equivalents for tix and eur), prefer:promo (promos), prefer:default (default Magic frame), prefer:atypical (atypical Magic frames), prefer:universesbeyond / prefer:ub (Universes Beyond prints), or prefer:notuniversesbeyond / prefer:notub (non-Universes Beyond prints).
Change the order of the sorted data with direction:asc or direction:desc.

Examples:

- `!"Lightning Bolt" unique:prints` — Every printing of Lightning Bolt
- `t:forest a:avon unique:art` — Every unique Forest illustration by John Avon
- `f:modern order:rarity direction:asc` — Modern legal cards sorted by rarity, commons first
- `t:human display:text` — Every Human card as text-only
- `in:leb game:paper prefer:newest` — The newest paper printing of each card in Limited Edition Beta
- `year=2025 prefer:atypical` — All cards printed in 2025, preferring atypical frame printings
