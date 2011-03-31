dojo.provide("sketchSpaceDesigner.designer.modes.Zoom");

dojo.require("sketchSpaceDesigner.designer.modes.Mode");

dojo.declare("sketchSpaceDesigner.designer.modes.Zoom", [sketchSpaceDesigner.designer.modes.Mode], {
  zoomFactor: 0.15,
  mouseFactor: 0.3,
  enable: function () {
    this.inherited(arguments);
    var mode = this;
  },
  disable: function () {
    this.inherited(arguments);
  },
  onMouseWheel: function (event, scroll) {
    this.inherited(arguments);
    scroll *= this.mouseFactor;
    if (scroll < 0)
      scroll = 1.0 / (1.0 - this.zoomFactor * scroll);
    else
      scroll = 1.0 + this.zoomFactor * scroll;
    this.onZoom(scroll, event.layerX, event.layerY);
  },
  onKeyUp: function (event) {
    this.inherited(arguments);
    if (event.keyCode == 38 && event.ctrlKey && !event.altKey && !event.shiftKey) {
      this.onZoom(1.0 + this.zoomFactor);
    } else if (event.keyCode == 40 && event.ctrlKey && !event.altKey && !event.shiftKey) {
      this.onZoom(1.0 / (1.0 + this.zoomFactor));
    }
  },
  onMouseDown: function(event) {
    this.inherited(arguments);
    this.designer.surface_transform.originalMatrix = this.designer.surface_transform.getTransform();
  },
  onMouseMove: function(event) {
    this.inherited(arguments);
    mouseDown = this.inputState.mouse[1];
    if (mouseDown != undefined && !mouseDown.ctrlKey && !mouseDown.altKey && !mouseDown.shiftKey) {
       var orig = this.getCurrentMouse(mouseDown, this.designer.surface);
       var mouse = this.getCurrentMouse(event, this.designer.surface);
       var move = dojox.gfx.matrix.translate(mouse.x - orig.x, mouse.y - orig.y);
       this.designer.surface_transform.setTransform(dojox.gfx.matrix.multiply(move, this.designer.surface_transform.originalMatrix));
    }
  },
  onZoom: function (zoom, x, y) {
    if (x === undefined) x = this.designer.surface_size.width / 2;
    if (y === undefined) y = this.designer.surface_size.height / 2;

    var screenToCurrentZoomMatrix = dojox.gfx.matrix.invert(this.designer.surface_transform._getRealMatrix());

    var mouse = dojox.gfx.matrix.multiplyPoint(screenToCurrentZoomMatrix, x, y);
    this.designer.surface_transform.applyTransform(dojox.gfx.matrix.scaleAt(zoom, zoom, mouse.x, mouse.y));
  },
});
