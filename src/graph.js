import { DeepMap, DeepSet } from "deep-equality-data-structures";

/**
 * @enum {number}
 */
var ElementKind = {
    INPUT: 1,
    INTERMEDIATE: 2,
    OUTPUT: 3,
};

/**
 * @typedef {{ id: number, kind: ElementKind } } Element
 */

const mask64 = 0xffffffffffffffffn;

export class Rng {
    /**
     * @param {BigInt} seed
     */
    constructor(seed) {
        this.seed_x = seed & mask64;
        this.seed_y = (seed << 1n) & mask64;
    }

    /**
     * @returns {BigInt} random number
     */
    get_random() {
        let x = this.seed_x;
        const y = this.seed_y;
        this.seed_x = this.seed_y;
        x ^= (x << 23n) & mask64;
        x ^= x >> 17n;
        x ^= y;
        this.seed_y = (x + y) & mask64;
        return x;
    }
}

/**
 * @param {number} start
 * @param {number} end
 * @returns {number[]}
 */
function range(start, end) {
    return [...Array(end - start + 1)].map((_, i) => start + i);
}

/**
 * @param {bigint} seed
 * @param {number} inputs
 * @param {number} outputs
 * @param {number} intermediates
 * @param {number} start_items
 * @param {any} compounds_are_ingredients
 *
 * @returns {{ recipes: DeepMap<[Element, Element], Element>, ingredients: Element[] }}
 */
export function create_graph(
    seed,
    inputs,
    outputs,
    intermediates,
    start_items,
    compounds_are_ingredients,
) {
    const dag_edges = [];
    const compound_edges = [];
    const already_used = new DeepSet();
    const rng = new Rng(seed);

    const inputs_to_place = range(1, inputs + 1);
    const intermediates_to_place = range(1, intermediates + 1);
    const outputs_to_place = range(1, outputs + 1);

    for (let i = 1; i <= start_items; i++) {
        dag_edges.push([-1, -1, i, 0]);
        inputs_to_place.splice(0, 1);
    }

    let inputs_placed = 0;
    let outputs_placed = 0;
    let to_place_length = inputs_to_place.length +
        intermediates_to_place.length +
        outputs_to_place.length;

    while (to_place_length > 0) {
        const previous_items = dag_edges.length;
        const new_layer = [];
        const max_layer_size = Math.min(
            Math.floor((previous_items * previous_items) / 2) -
            Math.floor(already_used.size / 2) -
            1,
            to_place_length - 1,
        );

        let new_layer_size = 1;
        if (max_layer_size > 0) {
            new_layer_size = Number(rng.get_random() % BigInt(max_layer_size)) +
                1;
        }

        for (let i = 0; i < new_layer_size; i++) {
            let to_place_type = -1;
            let attempts = 0;
            while (to_place_type == -1) {
                attempts = attempts + 1;
                const kind = Number(rng.get_random() % 3n) + 1;
                if (
                    kind == ElementKind.INPUT &&
                    outputs_placed > inputs_placed &&
                    inputs_to_place.length > 0
                ) {
                    to_place_type = 0;
                    inputs_placed = inputs_placed + 1;
                } else if (
                    kind == ElementKind.INTERMEDIATE &&
                    intermediates_to_place.length > 0
                ) {
                    to_place_type = 1;
                } else if (
                    kind == ElementKind.OUTPUT && outputs_to_place.length > 0
                ) {
                    to_place_type = 2;
                    outputs_placed = outputs_placed + 1;
                }
                if (attempts > 100) {
                    throw "Fuck you";
                }
            }
            let inputs1_idx = 0;
            let inputs2_idx = 0;
            do {
                inputs1_idx = Number(rng.get_random() % BigInt(previous_items));
                inputs2_idx = Number(rng.get_random() % BigInt(previous_items));
                if (inputs1_idx > inputs2_idx) {
                    [inputs1_idx, inputs2_idx] = [inputs2_idx, inputs1_idx];
                }
            } while (already_used.has([inputs1_idx, inputs2_idx]));

            already_used.add([inputs1_idx, inputs2_idx]);
            already_used.add([inputs2_idx, inputs1_idx]);

            let output;
            if (to_place_type == 0) {
                const output_idx = Number(
                    rng.get_random() % BigInt(inputs_to_place.length),
                );
                output = inputs_to_place.splice(output_idx, 1)[0];
            } else if (to_place_type == 1) {
                const output_idx = Number(
                    rng.get_random() % BigInt(intermediates_to_place.length),
                );
                output = intermediates_to_place.splice(output_idx, 1)[0];
            } else if (to_place_type == 2) {
                const output_idx = Number(
                    rng.get_random() % BigInt(outputs_to_place.length),
                );
                output = outputs_to_place.splice(output_idx, 1)[0];
            }

            if (compounds_are_ingredients || to_place_type != 2) {
                new_layer.push([
                    inputs1_idx,
                    inputs2_idx,
                    output,
                    to_place_type,
                ]);
            } else {
                compound_edges.push([
                    inputs1_idx,
                    inputs2_idx,
                    output,
                    to_place_type,
                ]);
            }
        }
        to_place_length = inputs_to_place.length +
            intermediates_to_place.length +
            outputs_to_place.length;

        dag_edges.push(...new_layer);
    }
    dag_edges.push(...compound_edges);

    const recipes_with_outputs = new DeepMap();
    for (const edge of dag_edges) {
        const [i1, i2, out, kind] = edge;
        if (kind == ElementKind.INPUT) continue;

        if (i1 < 0 && i2 < 0) continue;

        const to_insert_in = [
            { id: dag_edges[i1][2], kind: dag_edges[i1][3] },
            { id: dag_edges[i2][2], kind: dag_edges[i2][3] },
        ];
        const to_insert_out = { id: out, kind: kind };

        if (recipes_with_outputs.has(to_insert_in)) {
            recipes_with_outputs.get(to_insert_in).push(to_insert_out);
        } else {
            recipes_with_outputs.set(to_insert_in, [to_insert_out]);
        }
    }

    const all_ingredients = range(1, inputs + 1)
        .map((x) =>
            Object({
                id: x,
                kind: ElementKind.INPUT,
            })
        )
        .concat(
            range(1, intermediates + 1).map((x) =>
                Object({
                    id: x,
                    kind: ElementKind.INTERMEDIATE,
                })
            ),
        );

    if (compounds_are_ingredients) {
        all_ingredients.concat(
            range(1, outputs + 1).map((x) =>
                Object({
                    id: x,
                    kind: ElementKind.OUTPUT,
                })
            ),
        );
    }

    return {
        recipes: recipes_with_outputs,
        ingredients: all_ingredients,
    };
}
