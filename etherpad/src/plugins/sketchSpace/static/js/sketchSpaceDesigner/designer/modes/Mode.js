dojo.provide("sketchSpaceDesigner.designer.modes.Mode");

dojo.require("dojox.gfx.matrix");
dojo.require("dijit.layout.ContentPane");
dojo.require("sketchSpaceDesigner.designer.widgets");

dojo.declare("sketchSpaceDesigner.designer.modes.Mode", [], {
  constructor: function () {
    this.inputState = {};
    this.inputState.keyboard = {};
    this.inputState.mouse = {};
  },
  enable: function () {
    var mode = this;
    this.designer.forEachObjectShape(function (shape) { mode.enableShape(shape); });
    this.onMouseWheelHandle = dojo.connect(this.designer.container, (!dojo.isMozilla ? "onmousewheel" : "DOMMouseScroll"), function(e){
      var scroll = e[(!dojo.isMozilla ? "wheelDelta" : "detail")] * (!dojo.isMozilla ? 1 : -1);
      mode.onMouseWheel(e, scroll);
    });
    this.onKeyDownHandle = dojo.connect(document, "onkeydown", this, function (event) { mode.onKeyDown(event); });
    this.onKeyUpHandle = dojo.connect(document, "onkeyup", this, function (event) { mode.onKeyUp(event); });
    this.onMouseDownHandle = dojo.connect(this.designer.container, "onmousedown", this, this.onMouseDown);
    this.onMouseUpHandle = dojo.connect(this.designer.container, "onmouseup", this, this.onMouseUp);
    this.onMouseMoveHandle = dojo.connect(this.designer.container, "onmousemove", this, this.onMouseMove);
    this.onActivateHandle = dojo.connect(this.designer.container, "activate", this, this.onActivate);
    this.onContextMenuHandle = dojo.connect(this.designer.container, "contextmenu", this, this.onContextMenu);
    this.setOptionsHandle = dojo.connect(this.designer, "setOptions", this, this.onSetOptions);

    this.showAuthorshipColorOption = new sketchSpaceDesigner.designer.widgets.OptionCheckBox({title:"Show authorship:", optionsPath:"showAuthorshipColors", designer:this.designer});
    this.designer.ui.options.addChild(this.showAuthorshipColorOption);

    this.strokeColorPicker = new dijit.layout._LayoutWidget({title:"Stroke:"});
    this.strokeColorPicker.addChild(new sketchSpaceDesigner.designer.widgets.OptionCheckBox({optionsPath:"doStroke", designer:this.designer}));
    this.strokeColorPicker.addChild(new sketchSpaceDesigner.designer.widgets.ColorOptionInput({optionsPath:"stroke.color", designer:this.designer}));
    this.strokeColorPicker.addChild(new sketchSpaceDesigner.designer.widgets.OptionNumberSpinner({optionsPath:"stroke.width", designer:this.designer, style:"width:30pt"}));
    this.designer.ui.options.addChild(this.strokeColorPicker);

    this.fillColorPicker = new dijit.layout._LayoutWidget({title:"Fill:"});
    this.fillColorPicker.addChild(new sketchSpaceDesigner.designer.widgets.OptionCheckBox({optionsPath:"doFill", designer:this.designer}));
    this.fillColorPicker.addChild(new sketchSpaceDesigner.designer.widgets.ColorOptionInput({title:"Fill:", optionsPath:"fill", designer:this.designer}));
    this.designer.ui.options.addChild(this.fillColorPicker);

    this.designer.ui.options.layout();
  },
  disable: function () {
    var mode = this;
    dojo.disconnect(this.setOptionsHandle);
    dojo.disconnect(this.onContextMenuHandle);
    dojo.disconnect(this.onActivateHandle);
    dojo.disconnect(this.onMouseDownHandle);
    dojo.disconnect(this.onMouseUpHandle);
    dojo.disconnect(this.onMouseMoveHandle);
    dojo.disconnect(this.onKeyUpHandle);
    dojo.disconnect(this.onMouseWheelHandle);
    this.designer.forEachObjectShape(function (shape) { mode.disableShape(shape); });
    this.showAuthorshipColorOption.destroyRecursive();
    this.strokeColorPicker.destroyRecursive();
    this.fillColorPicker.destroyRecursive();
    this.designer.ui.options.layout();
  },
  enableShape: function (shape) {
    var mode = this;
    shape.onMouseUpHandle = shape.connect("onmouseup", shape, function (event) { mode.onShapeMouseUp(shape, event); });
    shape.onMouseDownHandle = shape.connect("onmousedown", shape, function (event) { mode.onShapeMouseDown(shape, event); });
  },
  disableShape: function (shape) {
    dojo.disconnect(shape.onMouseUpHandle);
    dojo.disconnect(shape.onMouseDownHandle);
  },
  onActivate: function  (event) { dojo.stopEvent(event); },
  onContextMenu: function  (event) { dojo.stopEvent(event); },
  onSetOptions: function () {},
  onMouseWheel: function (event, scroll) {
  },
  onKeyDown: function (event) {
    this.inputState.keyboard[event.keyCode] = event;
  },
  onKeyUp: function (event) {
    console.log([event.keyCode, event]);
    if (event.keyCode == 83) { /* key=s */
      this.designer.setOptions({doStroke: !this.designer.options.doStroke});
    } else if (event.keyCode == 70) { /* key=f */
      this.designer.setOptions({doFill: !this.designer.options.doFill});
    }
    delete this.inputState.keyboard[event.keyCode];
  },
  onMouseDown: function(event) {
    this.inputState.mouse[event.button] = event;
  },
  onMouseUp: function(event) {
    delete this.inputState.mouse[event.button];
  },
  onMouseMove: function(event) {
  },
  onShapeMouseDown: function (shape, event) {
  },
  onShapeMouseUp: function (shape, event) {
  },

  screenToLocalCoord: function (p, container) {
    if (container === undefined) container = this.getContainerShape();
    var screenToObjMatrix = dojox.gfx.matrix.invert(container._getRealMatrix());
    return dojox.gfx.matrix.multiplyPoint(screenToObjMatrix, p.x, p.y);
  },

  getCurrentMouse: function (event, container) {
   return this.screenToLocalCoord({x:event.layerX, y:event.layerY}, container);
  },

  getCurrentMove: function (event, container, orig) {
    var mouse = this.getCurrentMouse(event, container);
    if (orig === undefined) orig = this.orig;
    return dojox.gfx.matrix.translate(mouse.x - orig.x, mouse.y - orig.y);
  },

});
