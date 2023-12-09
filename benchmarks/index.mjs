const { Suite } = globalThis.Benchmark ?? (await import('benchmark')).default;

let suiteOps = {
  parse: {},
  serialize: {}
};

const libs = (
  await Promise.all(
    [
      'serde-wasm-bindgen',
      'serde-json',
      'serde-wasm-bindgen-reftypes',
      'msgpack'
    ].map(async dir => {
      try {
        const impl = await import(`./pkg/${dir}/serde_wasm_bindgen_benches.js`);
        // await impl.default(); // Init Wasm
        return { name: dir, impl };
      } catch (err) {
        console.warn(err.toString());
        return null;
      }
    })
  )
).filter(Boolean);

let readFile, filter;

const isNode = typeof process !== 'undefined';

if (isNode) {
  const prettyBytes = (await import('pretty-bytes')).default;
  const brotliSize = await import('brotli-size');

  console.log('=== Sizes ===');
  for (let { name } of libs) {
    const [js, wasm] = [
      'serde_wasm_bindgen_benches.js',
      'serde_wasm_bindgen_benches_bg.wasm'
    ].map(file => prettyBytes(brotliSize.fileSync(`pkg/${name}/${file}`)));

    console.log(`${name}: JS = ${js}, Wasm = ${wasm}`);
  }
  console.log();

  const readFileImpl = (await import('fs/promises')).readFile;
  readFile = name => readFileImpl(name, 'utf8');

  filter = process.argv[2];
} else {
  readFile = name =>
    fetch(name).then(res => {
      if (!res.ok) {
        throw new Error(`Failed to fetch ${name}`);
      }
      return res.text();
    });

  filter = new URLSearchParams(location.search).get('filter');
}

filter = new RegExp(filter ?? '(?:)', 'i');

async function loadData(name) {
  return JSON.parse(await readFile(`./data/${name}.json`, 'utf8'));
}

const datasets = {
  Canada: await loadData('canada'),
  CitmCatalog: await loadData('citm_catalog'),
  Twitter: await loadData('twitter')
};

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

let csv = '';

for (let [op, suites] of Object.entries(suiteOps)) {
  console.group(op);
  for (let suite of Object.values(suites)) {
    console.group(suite.name);
    await new Promise((resolve, reject) => {
      suite
        .on('error', event => reject(event.target.error))
        .on('cycle', event => {
          console.log(event.target.toString());
          csv += `${op},${suite.name},${event.target.name},${event.target.hz}\n`;
        })
        .on('complete', resolve)
        .run({
          async: true
        });
    });
    console.groupEnd();
  }
  console.groupEnd();
}

if (isNode) {
  (await import('fs')).writeFileSync('results.csv', csv);
} else {
  let csvLink = document.createElement('a');
  csvLink.href = URL.createObjectURL(new Blob([csv], { type: 'text/csv' }));
  csvLink.download = 'results.csv';
  csvLink.textContent = 'Download CSV';
  document.body.append('Done! ', csvLink);
}
