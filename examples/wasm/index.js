import init, { save, load, clear } from './pkg/wasm_example.js';

document.addEventListener('DOMContentLoaded', async function() {
	await init();

	await load();

	document.querySelector('#save').addEventListener('click', async function(event) {
		event.preventDefault();
		await save();
	});

	document.querySelector('#reload').addEventListener('click', async function(event) {
		event.preventDefault();
		await load();
	});

	document.querySelector('#delete').addEventListener('click', async function(event) {
		event.preventDefault();
		await clear();
	});
});
