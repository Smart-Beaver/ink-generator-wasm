const wasm = require('../pkg');
const {Contract} = require("../pkg");

const c = new Contract("PSP22", "Test name", ["metadata"], '/api');

wasm?.start(c);



