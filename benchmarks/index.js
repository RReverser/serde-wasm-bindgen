'use strict';

const { Suite } = require('benchmark');
const benches = require('./pkg');

let parseSuite = new Suite('parse');
let serializeSuite = new Suite('serialize');

for (let input of ['canada', 'citm_catalog', 'twitter']) {
	const json = require(`./${input}.json`);
	const free = benches[`free_${input}`];

	for (const lib of ['serde_json', 'serde_wasm_bindgen']) {
		const parse = benches[`parse_${input}_with_${lib}`];
		parseSuite.add(`${input} x ${lib}`, () => free(parse(json)));

		const serialize = benches[`serialize_${input}_with_${lib}`];
		let parsed = parse(json);
		serializeSuite.add(`${input} x ${lib}`, () => serialize(parsed), {
			onComplete: () => free(parsed)
		});
	}
}

function runSuite(suite) {
	console.log('='.repeat(suite.name.length));
	console.log(suite.name);
	console.log('='.repeat(suite.name.length));

	suite
	.on('error', event => console.error(event.target.error))
	.on('cycle', event => console.log(event.target.toString()))
	.run();
}

runSuite(parseSuite);
runSuite(serializeSuite);
