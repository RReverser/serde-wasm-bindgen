import Benchmark from 'benchmark';
import { readdirSync, readFileSync } from 'fs';
import prettyBytes from 'pretty-bytes';
import { fileSync as brotliSize } from 'brotli-size';
import { createRequire } from 'module';
import { inspect } from 'util';

const { Suite } = Benchmark;
const require = createRequire(import.meta.url);

let suiteOps = {
  parse: {},
  serialize: {}
};

const libs = readdirSync('pkg').map(dir => ({
  name: dir,
  impl: require(`./pkg/${dir}`)
}));

console.log('=== Sizes ===');
for (let { name } of libs) {
  const [js, wasm] = [
    'serde_wasm_bindgen_benches.js',
    'serde_wasm_bindgen_benches_bg.wasm'
  ].map(file => prettyBytes(brotliSize(`pkg/${name}/${file}`)));

  console.log(`${name}: JS = ${js}, Wasm = ${wasm}`);
}
console.log();

function loadData(name) {
  return JSON.parse(readFileSync(`./data/${name}.json`, 'utf8'));
}

const datasets = {
  Canada: loadData('canada'),
  CitmCatalog: loadData('citm_catalog'),
  Twitter: loadData('twitter')
};

let filter = new RegExp(process.argv[2] || '(?:)');

for (let { name: libName, impl } of libs) {
  for (let [dataName, json] of Object.entries(datasets)) {
    let { parse } = impl[dataName];

    let parseBenchName = `parse ${dataName} ${libName}`;
    if (filter.test(parseBenchName)) {
      (suiteOps.parse[dataName] ??= new Suite(dataName)).add(libName, () =>
        parse(json).free()
      );
    }

    let serializeBenchName = `serialize ${dataName} ${libName}`;
    if (filter.test(serializeBenchName)) {
      let parsed = parse(json);
      (suiteOps.serialize[dataName] ??= new Suite(dataName)).add(
        libName,
        () => parsed.serialize(),
        {
          onComplete: () => parsed.free()
        }
      );
    }
  }
}

console.log('=== Benchmarks ===');

for (let [op, suites] of Object.entries(suiteOps)) {
  console.group(op);
  for (let suite of Object.values(suites)) {
    console.group(suite.name);
    suite
      .on('error', event => console.error(event.target.error))
      .on('cycle', event => console.log(event.target.toString()))
      .run();
    console.groupEnd();
  }
  console.groupEnd();
}
