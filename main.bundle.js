/*
 * ATTENTION: The "eval" devtool has been used (maybe by default in mode: "development").
 * This devtool is not neither made for production nor for readable output files.
 * It uses "eval()" calls to create a separate source file in the browser devtools.
 * If you are trying to read the output file, select a different devtool (https://webpack.js.org/configuration/devtool/)
 * or disable the default devtool with "devtool: false".
 * If you are looking for production-ready output files, see mode: "production" (https://webpack.js.org/configuration/mode/).
 */
/******/ (() => { // webpackBootstrap
/******/ 	"use strict";
/******/ 	var __webpack_modules__ = ({

/***/ "./src/index.ts":
/*!**********************!*
  !*** ./src/index.ts ***!
  \**********************/
/***/ ((module, __webpack_exports__, __webpack_require__) => {

eval("module.exports = (async () => {\n__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"App\": () => /* binding */ App\n/* harmony export */ });\n/* harmony import */ var _babylonjs_core_Debug_debugLayer__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! @babylonjs/core/Debug/debugLayer */ \"./node_modules/@babylonjs/core/Debug/debugLayer.js\");\n/* harmony import */ var _babylonjs_inspector__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! @babylonjs/inspector */ \"./node_modules/@babylonjs/inspector/babylon.inspector.bundle.max.js\");\n/* harmony import */ var _babylonjs_inspector__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(_babylonjs_inspector__WEBPACK_IMPORTED_MODULE_1__);\n/* harmony import */ var _babylonjs_loaders_glTF__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! @babylonjs/loaders/glTF */ \"./node_modules/@babylonjs/loaders/glTF/index.js\");\n/* harmony import */ var _babylonjs_core__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! @babylonjs/core */ \"./node_modules/@babylonjs/core/index.js\");\n/* harmony import */ var _world_WorldScene__WEBPACK_IMPORTED_MODULE_4__ = __webpack_require__(/*! ./world/WorldScene */ \"./src/world/WorldScene.ts\");\n_world_WorldScene__WEBPACK_IMPORTED_MODULE_4__ = await Promise.resolve(_world_WorldScene__WEBPACK_IMPORTED_MODULE_4__);\n\n\n\n\n\nconst { Terra } = await __webpack_require__.e(/*! import() */ \"crates_wasm_pkg_terra-wasm_js\").then(__webpack_require__.bind(__webpack_require__, /*! ./wasm */ \"../crates/wasm/pkg/terra-wasm.js\"));\nconst CANVAS_ID = \"game-canvas\";\n/**\n * Top-level game class\n */\nclass App {\n    constructor() {\n        const canvas = document.getElementById(CANVAS_ID);\n        if (!canvas) {\n            throw new Error(`Could not find canvas by ID: ${CANVAS_ID}`);\n        }\n        // Initialize Terra once, which will let us generate worlds\n        const terra = new Terra();\n        // initialize babylon scene and engine\n        const engine = new _babylonjs_core__WEBPACK_IMPORTED_MODULE_3__.Engine(canvas, true, { audioEngine: false }, false);\n        const scene = new _world_WorldScene__WEBPACK_IMPORTED_MODULE_4__.default(terra, engine);\n        // run the main render loop\n        engine.runRenderLoop(() => {\n            scene.render();\n        });\n    }\n}\nnew App();\n\nreturn __webpack_exports__;\n})();\n\n//# sourceURL=webpack://terra/./src/index.ts?");

/***/ }),

/***/ "./src/util.ts":
/*!*********************!*
  !*** ./src/util.ts ***!
  \*********************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"assertUnreachable\": () => /* binding */ assertUnreachable\n/* harmony export */ });\n/**\n * Assert that a point in the code is unreachable. Useful for making sure switch\n * statements are exhaustive.\n * @param x The value that should never occur\n */\n// eslint-disable-next-line @typescript-eslint/no-unused-vars\nfunction assertUnreachable(x) {\n    throw new Error(\"Didn't expect to get here\");\n}\n\n\n//# sourceURL=webpack://terra/./src/util.ts?");

/***/ }),

/***/ "./src/world/InputHandler.ts":
/*!***********************************!*
  !*** ./src/world/InputHandler.ts ***!
  \***********************************/
/***/ ((module, __webpack_exports__, __webpack_require__) => {

eval("module.exports = (async () => {\n__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"default\": () => __WEBPACK_DEFAULT_EXPORT__\n/* harmony export */ });\n/* harmony import */ var _babylonjs_core__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! @babylonjs/core */ \"./node_modules/@babylonjs/core/index.js\");\n/* harmony import */ var _util__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ../util */ \"./src/util.ts\");\n\n\nconst config = await __webpack_require__.e(/*! import() */ \"src_input_json\").then(__webpack_require__.t.bind(__webpack_require__, /*! ../input.json */ \"./src/input.json\", 19));\nconst { TileLens } = await __webpack_require__.e(/*! import() */ \"crates_wasm_pkg_terra-wasm_js\").then(__webpack_require__.bind(__webpack_require__, /*! ../wasm */ \"../crates/wasm/pkg/terra-wasm.js\"));\nconst INPUT_ACTIONS = [\n    \"pause\",\n    \"toggleDebugOverlay\",\n    \"lensBiome\",\n    \"lensElevation\",\n    \"lensHumidity\",\n    \"lensRunoff\",\n];\nfunction isInputAction(s) {\n    return INPUT_ACTIONS.includes(s);\n}\nconst DEFAULT_INPUT_CONFIG = {\n    bindings: {\n        pause: \"ESCAPE\",\n        toggleDebugOverlay: \"`\",\n        lensBiome: \"1\",\n        lensElevation: \"2\",\n        lensHumidity: \"3\",\n        lensRunoff: \"4\",\n    },\n};\nclass InputHandler {\n    constructor(scene) {\n        this.config = Object.assign(Object.assign(Object.assign({}, DEFAULT_INPUT_CONFIG), config), { bindings: Object.assign(Object.assign({}, DEFAULT_INPUT_CONFIG.bindings), config === null || config === void 0 ? void 0 : config.bindings) });\n        this.scene = scene;\n        this.keyToEvent = new Map();\n        Object.entries(this.config.bindings).forEach(([key, value]) => {\n            // We could potentially get garbage actions from the user's config, so\n            // validate each action here\n            if (isInputAction(key)) {\n                this.keyToEvent.set(value.toUpperCase(), key);\n            }\n            else {\n                // eslint-disable-next-line no-console\n                console.warn(\"Unknown input action:\", key);\n            }\n        });\n    }\n    handleKeyEvent(kbInfo) {\n        if (kbInfo.type === _babylonjs_core__WEBPACK_IMPORTED_MODULE_0__.KeyboardEventTypes.KEYDOWN) {\n            // Map the keyboard key to a known action\n            const action = this.keyToEvent.get(kbInfo.event.key.toUpperCase());\n            if (action) {\n                this.handleAction(action);\n            }\n        }\n    }\n    handleAction(action) {\n        switch (action) {\n            case \"pause\":\n                this.scene.setPaused(true);\n                break;\n            case \"toggleDebugOverlay\":\n                this.scene.toggleDebugOverlay();\n                break;\n            case \"lensBiome\":\n                this.scene.setTileLens(TileLens.Biome);\n                break;\n            case \"lensElevation\":\n                this.scene.setTileLens(TileLens.Elevation);\n                break;\n            case \"lensHumidity\":\n                this.scene.setTileLens(TileLens.Humidity);\n                break;\n            case \"lensRunoff\":\n                this.scene.setTileLens(TileLens.Runoff);\n                break;\n            // Make sure this switch is exhaustive\n            default:\n                (0,_util__WEBPACK_IMPORTED_MODULE_1__.assertUnreachable)(action);\n        }\n    }\n}\n/* harmony default export */ const __WEBPACK_DEFAULT_EXPORT__ = (InputHandler);\n\nreturn __webpack_exports__;\n})();\n\n//# sourceURL=webpack://terra/./src/world/InputHandler.ts?");

/***/ }),

/***/ "./src/world/PauseMenu.ts":
/*!********************************!*
  !*** ./src/world/PauseMenu.ts ***!
  \********************************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"default\": () => __WEBPACK_DEFAULT_EXPORT__\n/* harmony export */ });\n/* harmony import */ var _babylonjs_core__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! @babylonjs/core */ \"./node_modules/@babylonjs/core/index.js\");\n/* harmony import */ var _babylonjs_gui__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! @babylonjs/gui */ \"./node_modules/@babylonjs/gui/index.js\");\n\n\n/**\n * In-game pause menu\n */\nclass PauseMenu {\n    constructor(engine, worldScene) {\n        this.scene = new _babylonjs_core__WEBPACK_IMPORTED_MODULE_0__.Scene(engine);\n        new _babylonjs_core__WEBPACK_IMPORTED_MODULE_0__.UniversalCamera(\"camera\", new _babylonjs_core__WEBPACK_IMPORTED_MODULE_0__.Vector3(0, 0, 0), this.scene);\n        this.texture = _babylonjs_gui__WEBPACK_IMPORTED_MODULE_1__.AdvancedDynamicTexture.CreateFullscreenUI(\"worldGenMenu\", true, this.scene);\n        const xmlLoader = new _babylonjs_gui__WEBPACK_IMPORTED_MODULE_1__.XmlLoader();\n        xmlLoader.loadLayout(\"/gui/pause.xml\", this.texture, () => {\n            xmlLoader.getNodeById(\"unpause\").onPointerUpObservable.add(() => {\n                worldScene.setPaused(false);\n            });\n        });\n    }\n    render() {\n        this.scene.render();\n    }\n}\n/* harmony default export */ const __WEBPACK_DEFAULT_EXPORT__ = (PauseMenu);\n\n\n//# sourceURL=webpack://terra/./src/world/PauseMenu.ts?");

/***/ }),

/***/ "./src/world/WorldRenderer.ts":
/*!************************************!*
  !*** ./src/world/WorldRenderer.ts ***!
  \************************************/
/***/ ((module, __webpack_exports__, __webpack_require__) => {

eval("module.exports = (async () => {\n__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"default\": () => __WEBPACK_DEFAULT_EXPORT__\n/* harmony export */ });\n/* harmony import */ var _babylonjs_core__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! @babylonjs/core */ \"./node_modules/@babylonjs/core/index.js\");\n\nconst { TileLens } = await __webpack_require__.e(/*! import() */ \"crates_wasm_pkg_terra-wasm_js\").then(__webpack_require__.bind(__webpack_require__, /*! ../wasm */ \"../crates/wasm/pkg/terra-wasm.js\"));\n/**\n * The length of one side of each tile. This is also the center-to-vertex\n * radius, because each tile is 6 equilateral triangles.\n */\nconst TILE_SIDE_LENGTH = 1.0;\n/**\n * Distance between two opposite vertices.\n */\nconst TILE_VERTEX_DIAM = TILE_SIDE_LENGTH * 2;\n/**\n * Util class for rendering a world of tiles\n */\nclass WorldRenderer {\n    constructor(scene, world) {\n        // We use \"thin instances\" here for the tiles cause #performance\n        // https://doc.babylonjs.com/divingDeeper/mesh/copies/thinInstances\n        // TODO there's a section on that page called \"Faster thin instances\", use\n        // that to speed up initialization\n        this.mesh = _babylonjs_core__WEBPACK_IMPORTED_MODULE_0__.MeshBuilder.CreateCylinder(\"tile\", {\n            diameter: TILE_VERTEX_DIAM,\n            tessellation: 6,\n            cap: _babylonjs_core__WEBPACK_IMPORTED_MODULE_0__.Mesh.CAP_END,\n        }, scene);\n        this.mesh.convertToUnIndexedMesh();\n        this.mesh.thinInstanceRegisterAttribute(\"color\", 4);\n        // This call allocates a whole new array, so we store the array instead of\n        // the full world object.\n        const tiles = world.wasm_tiles();\n        this.tiles = tiles.map((tile, i) => {\n            // Convert hex coords to pixel coords\n            // https://www.redblobgames.com/grids/hexagons/#coordinates-cube\n            const pos = tile.pos;\n            const transformMatrix = _babylonjs_core__WEBPACK_IMPORTED_MODULE_0__.Matrix.Translation(pos.x * 0.75 * TILE_VERTEX_DIAM, tile.height[0], (pos.x / 2 + pos.y) * -(Math.sqrt(3) / 2) * TILE_VERTEX_DIAM\n            // I'm not entirely sure why this scaling works, but it does\n            ).add(_babylonjs_core__WEBPACK_IMPORTED_MODULE_0__.Matrix.Scaling(0, tile.height[0], 0));\n            // Refresh meshes if this is the last tile in the list\n            const isLastTile = i === tiles.length - 1;\n            const idx = this.mesh.thinInstanceAdd(transformMatrix, isLastTile);\n            return [tile, idx];\n        });\n        this.tileLens = TileLens.Biome;\n        this.updateTileColors(this.tileLens);\n    }\n    updateTileColors(lens) {\n        this.tileLens = lens;\n        this.tiles.forEach(([tile, instanceIdx], i) => {\n            const isLastTile = i === this.tiles.length - 1;\n            const color = tile.color(this.tileLens);\n            this.mesh.thinInstanceSetAttributeAt(\"color\", instanceIdx, [color.red, color.green, color.blue, 1.0], \n            // Refresh meshes if this is the last tile in the list\n            isLastTile);\n        });\n    }\n}\n/* harmony default export */ const __WEBPACK_DEFAULT_EXPORT__ = (WorldRenderer);\n\nreturn __webpack_exports__;\n})();\n\n//# sourceURL=webpack://terra/./src/world/WorldRenderer.ts?");

/***/ }),

/***/ "./src/world/WorldScene.ts":
/*!*********************************!*
  !*** ./src/world/WorldScene.ts ***!
  \*********************************/
/***/ ((module, __webpack_exports__, __webpack_require__) => {

eval("module.exports = (async () => {\n__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"default\": () => __WEBPACK_DEFAULT_EXPORT__\n/* harmony export */ });\n/* harmony import */ var _babylonjs_core__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! @babylonjs/core */ \"./node_modules/@babylonjs/core/index.js\");\n/* harmony import */ var _WorldRenderer__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ./WorldRenderer */ \"./src/world/WorldRenderer.ts\");\n/* harmony import */ var _InputHandler__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ./InputHandler */ \"./src/world/InputHandler.ts\");\n/* harmony import */ var _PauseMenu__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! ./PauseMenu */ \"./src/world/PauseMenu.ts\");\n([_InputHandler__WEBPACK_IMPORTED_MODULE_2__, _WorldRenderer__WEBPACK_IMPORTED_MODULE_1__] = await Promise.all([_InputHandler__WEBPACK_IMPORTED_MODULE_2__, _WorldRenderer__WEBPACK_IMPORTED_MODULE_1__]));\n\n\n\n\n// We'll let Rust enforce the correct type here\n// eslint-disable-next-line @typescript-eslint/no-explicit-any\nconst config = await __webpack_require__.e(/*! import() */ \"src_world_json\").then(__webpack_require__.t.bind(__webpack_require__, /*! ../world.json */ \"./src/world.json\", 19));\nfunction initScene(engine) {\n    // Init world scene\n    const scene = new _babylonjs_core__WEBPACK_IMPORTED_MODULE_0__.Scene(engine);\n    // do a bunch of shit to make it go zoomer fast\n    // (doesn't actually make much of a difference)\n    scene.animationsEnabled = false;\n    scene.texturesEnabled = false;\n    scene.proceduralTexturesEnabled = false;\n    scene.collisionsEnabled = false;\n    scene.physicsEnabled = false;\n    scene.fogEnabled = false;\n    scene.particlesEnabled = false;\n    scene.blockMaterialDirtyMechanism = true;\n    // Init the camera\n    const camera = new _babylonjs_core__WEBPACK_IMPORTED_MODULE_0__.ArcRotateCamera(\"camera\", 0, Math.PI / 4, 500.0, new _babylonjs_core__WEBPACK_IMPORTED_MODULE_0__.Vector3(0.0, 200.0, 0.0), scene);\n    camera.lowerRadiusLimit = 1.0;\n    camera.upperRadiusLimit = 500.0;\n    camera.panningSensibility = 100;\n    camera.attachControl(engine.getRenderingCanvas(), true);\n    // Init world lighting\n    new _babylonjs_core__WEBPACK_IMPORTED_MODULE_0__.HemisphericLight(\"lightSun\", new _babylonjs_core__WEBPACK_IMPORTED_MODULE_0__.Vector3(0, 1, 0), scene);\n    return scene;\n}\nfunction getWorldConfig() {\n    const queryParams = new URLSearchParams(window.location.search);\n    const cfg = Object.assign({ \n        // bullshit here to pick a random seed if we don't have one\n        seed: Math.round(Math.random() * Number.MAX_SAFE_INTEGER) }, config);\n    // This is shitty but it works for now\n    const addQueryParam = (param) => {\n        const val = queryParams.get(param);\n        const parsed = parseInt(val !== null && val !== void 0 ? val : \"\", 10);\n        if (Number.isFinite(parsed)) {\n            cfg[param] = parsed;\n        }\n    };\n    addQueryParam(\"seed\");\n    addQueryParam(\"radius\");\n    return cfg;\n}\n/**\n * The scene that handles everything in-game\n */\nclass WorldScene {\n    constructor(terra, engine) {\n        this.terra = terra;\n        // Init world scene\n        this.scene = initScene(engine);\n        // Generate the world\n        this.world = this.terra.generate_world(getWorldConfig());\n        this.worldRenderer = new _WorldRenderer__WEBPACK_IMPORTED_MODULE_1__.default(this.scene, this.world);\n        this.scene.freezeActiveMeshes();\n        this.inputHandler = new _InputHandler__WEBPACK_IMPORTED_MODULE_2__.default(this);\n        this.scene.onKeyboardObservable.add((kbInfo) => this.inputHandler.handleKeyEvent(kbInfo));\n        // Init pause menu\n        this.pauseMenu = new _PauseMenu__WEBPACK_IMPORTED_MODULE_3__.default(engine, this);\n        this.paused = false;\n    }\n    setPaused(paused) {\n        this.paused = paused;\n    }\n    toggleDebugOverlay() {\n        if (this.scene.debugLayer.isVisible()) {\n            this.scene.debugLayer.hide();\n        }\n        else {\n            this.scene.debugLayer.show();\n        }\n    }\n    setTileLens(lens) {\n        this.worldRenderer.updateTileColors(lens);\n    }\n    render() {\n        if (this.paused) {\n            this.pauseMenu.render();\n        }\n        else {\n            this.scene.render();\n        }\n    }\n}\n/* harmony default export */ const __WEBPACK_DEFAULT_EXPORT__ = (WorldScene);\n\nreturn __webpack_exports__;\n})();\n\n//# sourceURL=webpack://terra/./src/world/WorldScene.ts?");

/***/ })

/******/ 	});
/************************************************************************/
/******/ 	// The module cache
/******/ 	var __webpack_module_cache__ = {};
/******/ 	
/******/ 	// The require function
/******/ 	function __webpack_require__(moduleId) {
/******/ 		// Check if module is in cache
/******/ 		if(__webpack_module_cache__[moduleId]) {
/******/ 			return __webpack_module_cache__[moduleId].exports;
/******/ 		}
/******/ 		// Create a new module (and put it into the cache)
/******/ 		var module = __webpack_module_cache__[moduleId] = {
/******/ 			id: moduleId,
/******/ 			loaded: false,
/******/ 			exports: {}
/******/ 		};
/******/ 	
/******/ 		// Execute the module function
/******/ 		__webpack_modules__[moduleId].call(module.exports, module, module.exports, __webpack_require__);
/******/ 	
/******/ 		// Flag the module as loaded
/******/ 		module.loaded = true;
/******/ 	
/******/ 		// Return the exports of the module
/******/ 		return module.exports;
/******/ 	}
/******/ 	
/******/ 	// expose the modules object (__webpack_modules__)
/******/ 	__webpack_require__.m = __webpack_modules__;
/******/ 	
/******/ 	// expose the module cache
/******/ 	__webpack_require__.c = __webpack_module_cache__;
/******/ 	
/************************************************************************/
/******/ 	/* webpack/runtime/compat get default export */
/******/ 	(() => {
/******/ 		// getDefaultExport function for compatibility with non-harmony modules
/******/ 		__webpack_require__.n = (module) => {
/******/ 			var getter = module && module.__esModule ?
/******/ 				() => module['default'] :
/******/ 				() => module;
/******/ 			__webpack_require__.d(getter, { a: getter });
/******/ 			return getter;
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/create fake namespace object */
/******/ 	(() => {
/******/ 		var getProto = Object.getPrototypeOf ? (obj) => Object.getPrototypeOf(obj) : (obj) => obj.__proto__;
/******/ 		var leafPrototypes;
/******/ 		// create a fake namespace object
/******/ 		// mode & 1: value is a module id, require it
/******/ 		// mode & 2: merge all properties of value into the ns
/******/ 		// mode & 4: return value when already ns object
/******/ 		// mode & 16: return value when it's Promise-like
/******/ 		// mode & 8|1: behave like require
/******/ 		__webpack_require__.t = function(value, mode) {
/******/ 			if(mode & 1) value = this(value);
/******/ 			if(mode & 8) return value;
/******/ 			if(typeof value === 'object' && value) {
/******/ 				if((mode & 4) && value.__esModule) return value;
/******/ 				if((mode & 16) && typeof value.then === 'function') return value;
/******/ 			}
/******/ 			var ns = Object.create(null);
/******/ 			__webpack_require__.r(ns);
/******/ 			var def = {};
/******/ 			leafPrototypes = leafPrototypes || [null, getProto({}), getProto([]), getProto(getProto)];
/******/ 			for(var current = mode & 2 && value; typeof current == 'object' && !~leafPrototypes.indexOf(current); current = getProto(current)) {
/******/ 				Object.getOwnPropertyNames(current).forEach(key => def[key] = () => value[key]);
/******/ 			}
/******/ 			def['default'] = () => value;
/******/ 			__webpack_require__.d(ns, def);
/******/ 			return ns;
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/define property getters */
/******/ 	(() => {
/******/ 		// define getter functions for harmony exports
/******/ 		__webpack_require__.d = (exports, definition) => {
/******/ 			for(var key in definition) {
/******/ 				if(__webpack_require__.o(definition, key) && !__webpack_require__.o(exports, key)) {
/******/ 					Object.defineProperty(exports, key, { enumerable: true, get: definition[key] });
/******/ 				}
/******/ 			}
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/ensure chunk */
/******/ 	(() => {
/******/ 		__webpack_require__.f = {};
/******/ 		// This file contains only the entry chunk.
/******/ 		// The chunk loading function for additional chunks
/******/ 		__webpack_require__.e = (chunkId) => {
/******/ 			return Promise.all(Object.keys(__webpack_require__.f).reduce((promises, key) => {
/******/ 				__webpack_require__.f[key](chunkId, promises);
/******/ 				return promises;
/******/ 			}, []));
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/get javascript chunk filename */
/******/ 	(() => {
/******/ 		// This function allow to reference async chunks
/******/ 		__webpack_require__.u = (chunkId) => {
/******/ 			// return url for filenames based on template
/******/ 			return "" + chunkId + ".bundle.js";
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/global */
/******/ 	(() => {
/******/ 		__webpack_require__.g = (function() {
/******/ 			if (typeof globalThis === 'object') return globalThis;
/******/ 			try {
/******/ 				return this || new Function('return this')();
/******/ 			} catch (e) {
/******/ 				if (typeof window === 'object') return window;
/******/ 			}
/******/ 		})();
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/harmony module decorator */
/******/ 	(() => {
/******/ 		__webpack_require__.hmd = (module) => {
/******/ 			module = Object.create(module);
/******/ 			if (!module.children) module.children = [];
/******/ 			Object.defineProperty(module, 'exports', {
/******/ 				enumerable: true,
/******/ 				set: () => {
/******/ 					throw new Error('ES Modules may not assign module.exports or exports.*, Use ESM export syntax, instead: ' + module.id);
/******/ 				}
/******/ 			});
/******/ 			return module;
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/hasOwnProperty shorthand */
/******/ 	(() => {
/******/ 		__webpack_require__.o = (obj, prop) => Object.prototype.hasOwnProperty.call(obj, prop)
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/load script */
/******/ 	(() => {
/******/ 		var inProgress = {};
/******/ 		var dataWebpackPrefix = "terra:";
/******/ 		// loadScript function to load a script via script tag
/******/ 		__webpack_require__.l = (url, done, key) => {
/******/ 			if(inProgress[url]) { inProgress[url].push(done); return; }
/******/ 			var script, needAttach;
/******/ 			if(key !== undefined) {
/******/ 				var scripts = document.getElementsByTagName("script");
/******/ 				for(var i = 0; i < scripts.length; i++) {
/******/ 					var s = scripts[i];
/******/ 					if(s.getAttribute("src") == url || s.getAttribute("data-webpack") == dataWebpackPrefix + key) { script = s; break; }
/******/ 				}
/******/ 			}
/******/ 			if(!script) {
/******/ 				needAttach = true;
/******/ 				script = document.createElement('script');
/******/ 		
/******/ 				script.charset = 'utf-8';
/******/ 				script.timeout = 120;
/******/ 				if (__webpack_require__.nc) {
/******/ 					script.setAttribute("nonce", __webpack_require__.nc);
/******/ 				}
/******/ 				script.setAttribute("data-webpack", dataWebpackPrefix + key);
/******/ 				script.src = url;
/******/ 			}
/******/ 			inProgress[url] = [done];
/******/ 			var onScriptComplete = (prev, event) => {
/******/ 				// avoid mem leaks in IE.
/******/ 				script.onerror = script.onload = null;
/******/ 				clearTimeout(timeout);
/******/ 				var doneFns = inProgress[url];
/******/ 				delete inProgress[url];
/******/ 				script.parentNode && script.parentNode.removeChild(script);
/******/ 				doneFns && doneFns.forEach((fn) => fn(event));
/******/ 				if(prev) return prev(event);
/******/ 			}
/******/ 			;
/******/ 			var timeout = setTimeout(onScriptComplete.bind(null, undefined, { type: 'timeout', target: script }), 120000);
/******/ 			script.onerror = onScriptComplete.bind(null, script.onerror);
/******/ 			script.onload = onScriptComplete.bind(null, script.onload);
/******/ 			needAttach && document.head.appendChild(script);
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/make namespace object */
/******/ 	(() => {
/******/ 		// define __esModule on exports
/******/ 		__webpack_require__.r = (exports) => {
/******/ 			if(typeof Symbol !== 'undefined' && Symbol.toStringTag) {
/******/ 				Object.defineProperty(exports, Symbol.toStringTag, { value: 'Module' });
/******/ 			}
/******/ 			Object.defineProperty(exports, '__esModule', { value: true });
/******/ 		};
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/publicPath */
/******/ 	(() => {
/******/ 		var scriptUrl;
/******/ 		if (__webpack_require__.g.importScripts) scriptUrl = __webpack_require__.g.location + "";
/******/ 		var document = __webpack_require__.g.document;
/******/ 		if (!scriptUrl && document) {
/******/ 			if (document.currentScript)
/******/ 				scriptUrl = document.currentScript.src
/******/ 			if (!scriptUrl) {
/******/ 				var scripts = document.getElementsByTagName("script");
/******/ 				if(scripts.length) scriptUrl = scripts[scripts.length - 1].src
/******/ 			}
/******/ 		}
/******/ 		// When supporting browsers where an automatic publicPath is not supported you must specify an output.publicPath manually via configuration
/******/ 		// or pass an empty string ("") and set the __webpack_public_path__ variable from your code to use your own logic.
/******/ 		if (!scriptUrl) throw new Error("Automatic publicPath is not supported in this browser");
/******/ 		scriptUrl = scriptUrl.replace(/#.*$/, "").replace(/\?.*$/, "").replace(/\/[^\/]+$/, "/");
/******/ 		__webpack_require__.p = scriptUrl;
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/jsonp chunk loading */
/******/ 	(() => {
/******/ 		// no baseURI
/******/ 		
/******/ 		// object to store loaded and loading chunks
/******/ 		// undefined = chunk not loaded, null = chunk preloaded/prefetched
/******/ 		// Promise = chunk loading, 0 = chunk loaded
/******/ 		var installedChunks = {
/******/ 			"main": 0
/******/ 		};
/******/ 		
/******/ 		var deferredModules = [
/******/ 			["./src/index.ts","vendors"]
/******/ 		];
/******/ 		__webpack_require__.f.j = (chunkId, promises) => {
/******/ 				// JSONP chunk loading for javascript
/******/ 				var installedChunkData = __webpack_require__.o(installedChunks, chunkId) ? installedChunks[chunkId] : undefined;
/******/ 				if(installedChunkData !== 0) { // 0 means "already installed".
/******/ 		
/******/ 					// a Promise means "currently loading".
/******/ 					if(installedChunkData) {
/******/ 						promises.push(installedChunkData[2]);
/******/ 					} else {
/******/ 						if(true) { // all chunks have JS
/******/ 							// setup Promise in chunk cache
/******/ 							var promise = new Promise((resolve, reject) => {
/******/ 								installedChunkData = installedChunks[chunkId] = [resolve, reject];
/******/ 							});
/******/ 							promises.push(installedChunkData[2] = promise);
/******/ 		
/******/ 							// start chunk loading
/******/ 							var url = __webpack_require__.p + __webpack_require__.u(chunkId);
/******/ 							// create error before stack unwound to get useful stacktrace later
/******/ 							var error = new Error();
/******/ 							var loadingEnded = (event) => {
/******/ 								if(__webpack_require__.o(installedChunks, chunkId)) {
/******/ 									installedChunkData = installedChunks[chunkId];
/******/ 									if(installedChunkData !== 0) installedChunks[chunkId] = undefined;
/******/ 									if(installedChunkData) {
/******/ 										var errorType = event && (event.type === 'load' ? 'missing' : event.type);
/******/ 										var realSrc = event && event.target && event.target.src;
/******/ 										error.message = 'Loading chunk ' + chunkId + ' failed.\n(' + errorType + ': ' + realSrc + ')';
/******/ 										error.name = 'ChunkLoadError';
/******/ 										error.type = errorType;
/******/ 										error.request = realSrc;
/******/ 										installedChunkData[1](error);
/******/ 									}
/******/ 								}
/******/ 							};
/******/ 							__webpack_require__.l(url, loadingEnded, "chunk-" + chunkId);
/******/ 						} else installedChunks[chunkId] = 0;
/******/ 					}
/******/ 				}
/******/ 		};
/******/ 		
/******/ 		// no prefetching
/******/ 		
/******/ 		// no preloaded
/******/ 		
/******/ 		// no HMR
/******/ 		
/******/ 		// no HMR manifest
/******/ 		
/******/ 		var checkDeferredModules = () => {
/******/ 		
/******/ 		};
/******/ 		function checkDeferredModulesImpl() {
/******/ 			var result;
/******/ 			for(var i = 0; i < deferredModules.length; i++) {
/******/ 				var deferredModule = deferredModules[i];
/******/ 				var fulfilled = true;
/******/ 				for(var j = 1; j < deferredModule.length; j++) {
/******/ 					var depId = deferredModule[j];
/******/ 					if(installedChunks[depId] !== 0) fulfilled = false;
/******/ 				}
/******/ 				if(fulfilled) {
/******/ 					deferredModules.splice(i--, 1);
/******/ 					result = __webpack_require__(__webpack_require__.s = deferredModule[0]);
/******/ 				}
/******/ 			}
/******/ 			if(deferredModules.length === 0) {
/******/ 				__webpack_require__.x();
/******/ 				__webpack_require__.x = () => {
/******/ 		
/******/ 				}
/******/ 			}
/******/ 			return result;
/******/ 		}
/******/ 		__webpack_require__.x = () => {
/******/ 			// reset startup function so it can be called again when more startup code is added
/******/ 			__webpack_require__.x = () => {
/******/ 		
/******/ 			}
/******/ 			chunkLoadingGlobal = chunkLoadingGlobal.slice();
/******/ 			for(var i = 0; i < chunkLoadingGlobal.length; i++) webpackJsonpCallback(chunkLoadingGlobal[i]);
/******/ 			return (checkDeferredModules = checkDeferredModulesImpl)();
/******/ 		};
/******/ 		
/******/ 		// install a JSONP callback for chunk loading
/******/ 		var webpackJsonpCallback = (data) => {
/******/ 			var [chunkIds, moreModules, runtime, executeModules] = data;
/******/ 			// add "moreModules" to the modules object,
/******/ 			// then flag all "chunkIds" as loaded and fire callback
/******/ 			var moduleId, chunkId, i = 0, resolves = [];
/******/ 			for(;i < chunkIds.length; i++) {
/******/ 				chunkId = chunkIds[i];
/******/ 				if(__webpack_require__.o(installedChunks, chunkId) && installedChunks[chunkId]) {
/******/ 					resolves.push(installedChunks[chunkId][0]);
/******/ 				}
/******/ 				installedChunks[chunkId] = 0;
/******/ 			}
/******/ 			for(moduleId in moreModules) {
/******/ 				if(__webpack_require__.o(moreModules, moduleId)) {
/******/ 					__webpack_require__.m[moduleId] = moreModules[moduleId];
/******/ 				}
/******/ 			}
/******/ 			if(runtime) runtime(__webpack_require__);
/******/ 			parentChunkLoadingFunction(data);
/******/ 			while(resolves.length) {
/******/ 				resolves.shift()();
/******/ 			}
/******/ 		
/******/ 			// add entry modules from loaded chunk to deferred list
/******/ 			if(executeModules) deferredModules.push.apply(deferredModules, executeModules);
/******/ 		
/******/ 			// run deferred modules when all chunks ready
/******/ 			return checkDeferredModules();
/******/ 		}
/******/ 		
/******/ 		var chunkLoadingGlobal = self["webpackChunkterra"] = self["webpackChunkterra"] || [];
/******/ 		var parentChunkLoadingFunction = chunkLoadingGlobal.push.bind(chunkLoadingGlobal);
/******/ 		chunkLoadingGlobal.push = webpackJsonpCallback;
/******/ 	})();
/******/ 	
/******/ 	/* webpack/runtime/wasm chunk loading */
/******/ 	(() => {
/******/ 		// object to store loaded and loading wasm modules
/******/ 		var installedWasmModules = {};
/******/ 		
/******/ 		function promiseResolve() { return Promise.resolve(); }
/******/ 		
/******/ 		var wasmImportedFuncCache0;
/******/ 		var wasmImportedFuncCache1;
/******/ 		var wasmImportedFuncCache2;
/******/ 		var wasmImportedFuncCache3;
/******/ 		var wasmImportedFuncCache4;
/******/ 		var wasmImportedFuncCache5;
/******/ 		var wasmImportedFuncCache6;
/******/ 		var wasmImportedFuncCache7;
/******/ 		var wasmImportedFuncCache8;
/******/ 		var wasmImportedFuncCache9;
/******/ 		var wasmImportedFuncCache10;
/******/ 		var wasmImportedFuncCache11;
/******/ 		var wasmImportedFuncCache12;
/******/ 		var wasmImportedFuncCache13;
/******/ 		var wasmImportedFuncCache14;
/******/ 		var wasmImportedFuncCache15;
/******/ 		var wasmImportedFuncCache16;
/******/ 		var wasmImportedFuncCache17;
/******/ 		var wasmImportedFuncCache18;
/******/ 		var wasmImportedFuncCache19;
/******/ 		var wasmImportedFuncCache20;
/******/ 		var wasmImportedFuncCache21;
/******/ 		var wasmImportedFuncCache22;
/******/ 		var wasmImportedFuncCache23;
/******/ 		var wasmImportedFuncCache24;
/******/ 		var wasmImportedFuncCache25;
/******/ 		var wasmImportedFuncCache26;
/******/ 		var wasmImportedFuncCache27;
/******/ 		var wasmImportedFuncCache28;
/******/ 		var wasmImportedFuncCache29;
/******/ 		var wasmImportedFuncCache30;
/******/ 		var wasmImportedFuncCache31;
/******/ 		var wasmImportedFuncCache32;
/******/ 		var wasmImportedFuncCache33;
/******/ 		var wasmImportedFuncCache34;
/******/ 		var wasmImportedFuncCache35;
/******/ 		var wasmImportedFuncCache36;
/******/ 		var wasmImportedFuncCache37;
/******/ 		var wasmImportedFuncCache38;
/******/ 		var wasmImportedFuncCache39;
/******/ 		var wasmImportedFuncCache40;
/******/ 		var wasmImportedFuncCache41;
/******/ 		var wasmImportedFuncCache42;
/******/ 		var wasmImportedFuncCache43;
/******/ 		var wasmImportedFuncCache44;
/******/ 		var wasmImportObjects = {
/******/ 			"../crates/wasm/pkg/terra-wasm_bg.wasm": function() {
/******/ 				return {
/******/ 					"./terra-wasm_bg.js": {
/******/ 						"__wbindgen_object_drop_ref": function(p0i32) {
/******/ 							if(wasmImportedFuncCache0 === undefined) wasmImportedFuncCache0 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache0["__wbindgen_object_drop_ref"](p0i32);
/******/ 						},
/******/ 						"__wbindgen_string_new": function(p0i32,p1i32) {
/******/ 							if(wasmImportedFuncCache1 === undefined) wasmImportedFuncCache1 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache1["__wbindgen_string_new"](p0i32,p1i32);
/******/ 						},
/******/ 						"__wbindgen_object_clone_ref": function(p0i32) {
/******/ 							if(wasmImportedFuncCache2 === undefined) wasmImportedFuncCache2 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache2["__wbindgen_object_clone_ref"](p0i32);
/******/ 						},
/******/ 						"__wbindgen_is_null": function(p0i32) {
/******/ 							if(wasmImportedFuncCache3 === undefined) wasmImportedFuncCache3 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache3["__wbindgen_is_null"](p0i32);
/******/ 						},
/******/ 						"__wbindgen_is_undefined": function(p0i32) {
/******/ 							if(wasmImportedFuncCache4 === undefined) wasmImportedFuncCache4 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache4["__wbindgen_is_undefined"](p0i32);
/******/ 						},
/******/ 						"__wbg_new_59cb74e423758ede": function() {
/******/ 							if(wasmImportedFuncCache5 === undefined) wasmImportedFuncCache5 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache5["__wbg_new_59cb74e423758ede"]();
/******/ 						},
/******/ 						"__wbg_stack_558ba5917b466edd": function(p0i32,p1i32) {
/******/ 							if(wasmImportedFuncCache6 === undefined) wasmImportedFuncCache6 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache6["__wbg_stack_558ba5917b466edd"](p0i32,p1i32);
/******/ 						},
/******/ 						"__wbg_error_4bb6c2a97407129a": function(p0i32,p1i32) {
/******/ 							if(wasmImportedFuncCache7 === undefined) wasmImportedFuncCache7 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache7["__wbg_error_4bb6c2a97407129a"](p0i32,p1i32);
/******/ 						},
/******/ 						"__wbg_tile_new": function(p0i32) {
/******/ 							if(wasmImportedFuncCache8 === undefined) wasmImportedFuncCache8 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache8["__wbg_tile_new"](p0i32);
/******/ 						},
/******/ 						"__wbg_debug_9f067aefe2ceaadd": function(p0i32,p1i32,p2i32,p3i32) {
/******/ 							if(wasmImportedFuncCache9 === undefined) wasmImportedFuncCache9 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache9["__wbg_debug_9f067aefe2ceaadd"](p0i32,p1i32,p2i32,p3i32);
/******/ 						},
/******/ 						"__wbg_error_e325755affc8634b": function(p0i32) {
/******/ 							if(wasmImportedFuncCache10 === undefined) wasmImportedFuncCache10 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache10["__wbg_error_e325755affc8634b"](p0i32);
/******/ 						},
/******/ 						"__wbg_error_7bb15b842d5b0ddb": function(p0i32,p1i32,p2i32,p3i32) {
/******/ 							if(wasmImportedFuncCache11 === undefined) wasmImportedFuncCache11 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache11["__wbg_error_7bb15b842d5b0ddb"](p0i32,p1i32,p2i32,p3i32);
/******/ 						},
/******/ 						"__wbg_info_1b9fdabaafc8f4cb": function(p0i32,p1i32,p2i32,p3i32) {
/******/ 							if(wasmImportedFuncCache12 === undefined) wasmImportedFuncCache12 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache12["__wbg_info_1b9fdabaafc8f4cb"](p0i32,p1i32,p2i32,p3i32);
/******/ 						},
/******/ 						"__wbg_log_37120b26fb738792": function(p0i32,p1i32,p2i32,p3i32) {
/******/ 							if(wasmImportedFuncCache13 === undefined) wasmImportedFuncCache13 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache13["__wbg_log_37120b26fb738792"](p0i32,p1i32,p2i32,p3i32);
/******/ 						},
/******/ 						"__wbg_time_7b20e0fb24128e35": function(p0i32,p1i32) {
/******/ 							if(wasmImportedFuncCache14 === undefined) wasmImportedFuncCache14 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache14["__wbg_time_7b20e0fb24128e35"](p0i32,p1i32);
/******/ 						},
/******/ 						"__wbg_timeEnd_533927dc25d673d0": function(p0i32,p1i32) {
/******/ 							if(wasmImportedFuncCache15 === undefined) wasmImportedFuncCache15 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache15["__wbg_timeEnd_533927dc25d673d0"](p0i32,p1i32);
/******/ 						},
/******/ 						"__wbg_warn_6add4f04160cdbba": function(p0i32,p1i32,p2i32,p3i32) {
/******/ 							if(wasmImportedFuncCache16 === undefined) wasmImportedFuncCache16 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache16["__wbg_warn_6add4f04160cdbba"](p0i32,p1i32,p2i32,p3i32);
/******/ 						},
/******/ 						"__wbg_getRandomValues_c73f06b5ed8b878d": function(p0i32,p1i32) {
/******/ 							if(wasmImportedFuncCache17 === undefined) wasmImportedFuncCache17 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache17["__wbg_getRandomValues_c73f06b5ed8b878d"](p0i32,p1i32);
/******/ 						},
/******/ 						"__wbg_randomFillSync_5fa0a72035c7bfd9": function(p0i32,p1i32,p2i32) {
/******/ 							if(wasmImportedFuncCache18 === undefined) wasmImportedFuncCache18 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache18["__wbg_randomFillSync_5fa0a72035c7bfd9"](p0i32,p1i32,p2i32);
/******/ 						},
/******/ 						"__wbg_self_23b14d60c8dbf9da": function() {
/******/ 							if(wasmImportedFuncCache19 === undefined) wasmImportedFuncCache19 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache19["__wbg_self_23b14d60c8dbf9da"]();
/******/ 						},
/******/ 						"__wbg_static_accessor_MODULE_ff1e47f7076e0ee1": function() {
/******/ 							if(wasmImportedFuncCache20 === undefined) wasmImportedFuncCache20 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache20["__wbg_static_accessor_MODULE_ff1e47f7076e0ee1"]();
/******/ 						},
/******/ 						"__wbg_require_1dab18ea211c4fa1": function(p0i32,p1i32,p2i32) {
/******/ 							if(wasmImportedFuncCache21 === undefined) wasmImportedFuncCache21 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache21["__wbg_require_1dab18ea211c4fa1"](p0i32,p1i32,p2i32);
/******/ 						},
/******/ 						"__wbg_crypto_df96f3577c8a9bae": function(p0i32) {
/******/ 							if(wasmImportedFuncCache22 === undefined) wasmImportedFuncCache22 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache22["__wbg_crypto_df96f3577c8a9bae"](p0i32);
/******/ 						},
/******/ 						"__wbg_msCrypto_331efcdb9be40d7c": function(p0i32) {
/******/ 							if(wasmImportedFuncCache23 === undefined) wasmImportedFuncCache23 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache23["__wbg_msCrypto_331efcdb9be40d7c"](p0i32);
/******/ 						},
/******/ 						"__wbg_get_85e0a3b459845fe2": function(p0i32,p1i32) {
/******/ 							if(wasmImportedFuncCache24 === undefined) wasmImportedFuncCache24 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache24["__wbg_get_85e0a3b459845fe2"](p0i32,p1i32);
/******/ 						},
/******/ 						"__wbg_new_9dff83a08f5994f3": function() {
/******/ 							if(wasmImportedFuncCache25 === undefined) wasmImportedFuncCache25 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache25["__wbg_new_9dff83a08f5994f3"]();
/******/ 						},
/******/ 						"__wbg_push_3ddd8187ff2ff82d": function(p0i32,p1i32) {
/******/ 							if(wasmImportedFuncCache26 === undefined) wasmImportedFuncCache26 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache26["__wbg_push_3ddd8187ff2ff82d"](p0i32,p1i32);
/******/ 						},
/******/ 						"__wbg_instanceof_ArrayBuffer_3a0fa134e6809d57": function(p0i32) {
/******/ 							if(wasmImportedFuncCache27 === undefined) wasmImportedFuncCache27 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache27["__wbg_instanceof_ArrayBuffer_3a0fa134e6809d57"](p0i32);
/******/ 						},
/******/ 						"__wbg_new_94a7dfa9529ec6e8": function(p0i32,p1i32) {
/******/ 							if(wasmImportedFuncCache28 === undefined) wasmImportedFuncCache28 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache28["__wbg_new_94a7dfa9529ec6e8"](p0i32,p1i32);
/******/ 						},
/******/ 						"__wbg_isSafeInteger_ca75f5e5231bd3c7": function(p0i32) {
/******/ 							if(wasmImportedFuncCache29 === undefined) wasmImportedFuncCache29 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache29["__wbg_isSafeInteger_ca75f5e5231bd3c7"](p0i32);
/******/ 						},
/******/ 						"__wbg_buffer_3f12a1c608c6d04e": function(p0i32) {
/******/ 							if(wasmImportedFuncCache30 === undefined) wasmImportedFuncCache30 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache30["__wbg_buffer_3f12a1c608c6d04e"](p0i32);
/******/ 						},
/******/ 						"__wbg_new_c6c0228e6d22a2f9": function(p0i32) {
/******/ 							if(wasmImportedFuncCache31 === undefined) wasmImportedFuncCache31 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache31["__wbg_new_c6c0228e6d22a2f9"](p0i32);
/******/ 						},
/******/ 						"__wbg_set_b91afac9fd216d99": function(p0i32,p1i32,p2i32) {
/******/ 							if(wasmImportedFuncCache32 === undefined) wasmImportedFuncCache32 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache32["__wbg_set_b91afac9fd216d99"](p0i32,p1i32,p2i32);
/******/ 						},
/******/ 						"__wbg_length_c645e7c02233b440": function(p0i32) {
/******/ 							if(wasmImportedFuncCache33 === undefined) wasmImportedFuncCache33 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache33["__wbg_length_c645e7c02233b440"](p0i32);
/******/ 						},
/******/ 						"__wbg_instanceof_Uint8Array_fda7b6a64c667462": function(p0i32) {
/******/ 							if(wasmImportedFuncCache34 === undefined) wasmImportedFuncCache34 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache34["__wbg_instanceof_Uint8Array_fda7b6a64c667462"](p0i32);
/******/ 						},
/******/ 						"__wbg_newwithlength_a429e08f8a8fe4b3": function(p0i32) {
/******/ 							if(wasmImportedFuncCache35 === undefined) wasmImportedFuncCache35 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache35["__wbg_newwithlength_a429e08f8a8fe4b3"](p0i32);
/******/ 						},
/******/ 						"__wbg_subarray_02e2fcfa6b285cb2": function(p0i32,p1i32,p2i32) {
/******/ 							if(wasmImportedFuncCache36 === undefined) wasmImportedFuncCache36 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache36["__wbg_subarray_02e2fcfa6b285cb2"](p0i32,p1i32,p2i32);
/******/ 						},
/******/ 						"__wbg_byteLength_11e6bdc2fac53a3c": function(p0i32) {
/******/ 							if(wasmImportedFuncCache37 === undefined) wasmImportedFuncCache37 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache37["__wbg_byteLength_11e6bdc2fac53a3c"](p0i32);
/******/ 						},
/******/ 						"__wbindgen_number_get": function(p0i32,p1i32) {
/******/ 							if(wasmImportedFuncCache38 === undefined) wasmImportedFuncCache38 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache38["__wbindgen_number_get"](p0i32,p1i32);
/******/ 						},
/******/ 						"__wbindgen_string_get": function(p0i32,p1i32) {
/******/ 							if(wasmImportedFuncCache39 === undefined) wasmImportedFuncCache39 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache39["__wbindgen_string_get"](p0i32,p1i32);
/******/ 						},
/******/ 						"__wbindgen_boolean_get": function(p0i32) {
/******/ 							if(wasmImportedFuncCache40 === undefined) wasmImportedFuncCache40 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache40["__wbindgen_boolean_get"](p0i32);
/******/ 						},
/******/ 						"__wbindgen_debug_string": function(p0i32,p1i32) {
/******/ 							if(wasmImportedFuncCache41 === undefined) wasmImportedFuncCache41 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache41["__wbindgen_debug_string"](p0i32,p1i32);
/******/ 						},
/******/ 						"__wbindgen_throw": function(p0i32,p1i32) {
/******/ 							if(wasmImportedFuncCache42 === undefined) wasmImportedFuncCache42 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache42["__wbindgen_throw"](p0i32,p1i32);
/******/ 						},
/******/ 						"__wbindgen_rethrow": function(p0i32) {
/******/ 							if(wasmImportedFuncCache43 === undefined) wasmImportedFuncCache43 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache43["__wbindgen_rethrow"](p0i32);
/******/ 						},
/******/ 						"__wbindgen_memory": function() {
/******/ 							if(wasmImportedFuncCache44 === undefined) wasmImportedFuncCache44 = __webpack_require__.c["../crates/wasm/pkg/terra-wasm_bg.js"].exports;
/******/ 							return wasmImportedFuncCache44["__wbindgen_memory"]();
/******/ 						}
/******/ 					}
/******/ 				};
/******/ 			},
/******/ 		};
/******/ 		
/******/ 		var wasmModuleMap = {
/******/ 			"crates_wasm_pkg_terra-wasm_js": [
/******/ 				"../crates/wasm/pkg/terra-wasm_bg.wasm"
/******/ 			]
/******/ 		};
/******/ 		
/******/ 		// object with all WebAssembly.instance exports
/******/ 		__webpack_require__.w = {};
/******/ 		
/******/ 		// Fetch + compile chunk loading for webassembly
/******/ 		__webpack_require__.f.wasm = function(chunkId, promises) {
/******/ 		
/******/ 			var wasmModules = wasmModuleMap[chunkId] || [];
/******/ 		
/******/ 			wasmModules.forEach(function(wasmModuleId, idx) {
/******/ 				var installedWasmModuleData = installedWasmModules[wasmModuleId];
/******/ 		
/******/ 				// a Promise means "currently loading" or "already loaded".
/******/ 				if(installedWasmModuleData)
/******/ 					promises.push(installedWasmModuleData);
/******/ 				else {
/******/ 					var importObject = wasmImportObjects[wasmModuleId]();
/******/ 					var req = fetch(__webpack_require__.p + "" + {"crates_wasm_pkg_terra-wasm_js":{"../crates/wasm/pkg/terra-wasm_bg.wasm":"2b91920a6c3bb2d247d4"}}[chunkId][wasmModuleId] + ".module.wasm");
/******/ 					var promise;
/******/ 					if(importObject instanceof Promise && typeof WebAssembly.compileStreaming === 'function') {
/******/ 						promise = Promise.all([WebAssembly.compileStreaming(req), importObject]).then(function(items) {
/******/ 							return WebAssembly.instantiate(items[0], items[1]);
/******/ 						});
/******/ 					} else if(typeof WebAssembly.instantiateStreaming === 'function') {
/******/ 						promise = WebAssembly.instantiateStreaming(req, importObject);
/******/ 					} else {
/******/ 						var bytesPromise = req.then(function(x) { return x.arrayBuffer(); });
/******/ 						promise = bytesPromise.then(function(bytes) {
/******/ 							return WebAssembly.instantiate(bytes, importObject);
/******/ 						});
/******/ 					}
/******/ 					promises.push(installedWasmModules[wasmModuleId] = promise.then(function(res) {
/******/ 						return __webpack_require__.w[wasmModuleId] = (res.instance || res).exports;
/******/ 					}));
/******/ 				}
/******/ 			});
/******/ 		};
/******/ 	})();
/******/ 	
/************************************************************************/
/******/ 	// module cache are used so entry inlining is disabled
/******/ 	// run startup
/******/ 	return __webpack_require__.x();
/******/ })()
;