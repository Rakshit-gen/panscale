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
Flour, sugar, cocoa, butter and oats are weighed, using the grams per cup from
King Arthur Baking's weight chart. Spoons stay as spoons.

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
butter, weighs the flour, sugar and cocoa, and turns the oven to Celsius:

```
- 115 g butter
- 200 g sugar
- 2 eggs
- 1 tsp vanilla extract
- 28 g cocoa powder
- 60 g plain flour
- 1/4 tsp salt
- 1/4 tsp baking powder
- 85 g dark chocolate, chopped

## Method

1. Heat the oven to 175°C and line the pan.
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
- Only the handful of ingredients above are weighed. Anything else measured
  in cups comes out in ml.
- Apart from eggs, yolks and whites, the words after an amount aren't
  changed, so "1 lemons" can happen after scaling down.

## License

MIT
