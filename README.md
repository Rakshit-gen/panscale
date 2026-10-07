# panscale

Scales a recipe by servings or by pan size, and writes the result the way a
cook would measure it: 1 tbsp rather than 3 tsp, 1.5 kg rather than 1500 g,
whole eggs, and no 1 1/3 tbsp.

The recipe is plain text. Lines that start with an amount get scaled, the
"Serves 4" or "Makes 12" line gets updated, and everything under a Method,
Instructions or Directions heading is left as written.

## Use

```sh
cargo build --release
panscale scale recipe.txt --by 1.5
panscale scale recipe.txt --serves 6
panscale scale recipe.txt --from-pan "8in square" --pan 9x13
panscale pan "9in round" 9x13
```

The recipe can also come on stdin. `--metric` writes cups, fluid ounces,
ounces and pounds as millilitres and grams, and oven temperatures in Celsius.
Spoons stay as spoons.

The brownie recipe in `examples/` is written for an 8 inch square pan. Moving
it to a 9x13:

```
$ panscale scale examples/brownies.txt --from-pan "8in square" --pan 9x13
# Fudgy brownies

Makes 29 squares, in an 8in square pan.

## Ingredients

- 7/8 cup (210 g) butter
- 1 7/8 cups sugar
- 4 eggs
- 1 7/8 tsp vanilla extract
- 5/8 cup cocoa powder
- 7/8 cup plain flour
- 1/2 tsp salt
- 1/2 tsp baking powder
- 5 1/2 oz dark chocolate, chopped
```

The same recipe with `--by 1 --metric` uses the grams it already gives for the
butter and converts the rest:

```
- 115 g butter
- 235 ml sugar
- 2 eggs
- 1 tsp vanilla extract
- 79 ml cocoa powder
- 120 ml plain flour
- 1/4 tsp salt
- 1/4 tsp baking powder
- 85 g dark chocolate, chopped
```

```
$ panscale pan "8in round" "9in round"
9in round holds 1.27 times as much as 8in round.
```

## What it reads

- Amounts: `2`, `1.5`, `1/2`, `1 1/2`, `1½`, ranges like `2-3` and `2 to 3`.
- Units: tsp, tbsp, cup, fl oz, pint, quart, ml, l, g, kg, oz, lb, pinch and
  dash, with the usual spellings. `T` is a tablespoon and `t` a teaspoon.
- A second measure in brackets, like `1 cup (125 g)`, is scaled too.
- Pans: `9in round`, `23cm round`, `8" square`, `9x13`, `20x30cm`. Inches
  unless it says cm.

## Limits

- Pan scaling goes by base area and assumes the same depth. Baking times
  change with pan size and panscale doesn't touch them.
- US cup and spoon sizes. A metric cup (250 ml) reads as a US cup (237 ml).
- Volume to weight needs a density per ingredient, so `--metric` gives
  flour and sugar in ml, not grams.
- Text after the amount isn't changed, so "1 egg yolks" can happen after
  scaling down.

## License

MIT
