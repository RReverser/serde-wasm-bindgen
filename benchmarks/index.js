'use strict';

const { Suite } = require('benchmark');
const benches = require('./pkg');

const canada = require('./canada.json');

new Suite('canada')
.add('serde-json', () => benches.parse_canada_with_serde_json(canada))
.add('serde-wasm-bindgen', () => benches.parse_canada_with_serde_wasm_bindgen(canada))
.on('error', event => console.error(event.target.error))
.on('cycle', event => console.log(event.target.toString()))
.run();
