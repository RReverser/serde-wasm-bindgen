'use strict';

const { Suite } = require('benchmark');
const benches = require('./pkg');

let suite = new Suite();

for (let input of ['canada', 'citm_catalog', 'twitter']) {
	const json = require(`./${input}.json`);
	const free = benches[`free_${input}`];

	for (const lib of ['serde_json', 'serde_wasm_bindgen']) {
		const parse = benches[`parse_${input}_with_${lib}`];
		suite.add(`${input} x ${lib}`, () => free(parse(json)));
	}
}

suite
.on('error', event => console.error(event.target.error))
.on('cycle', event => console.log(event.target.toString()))
.run();
