dojo.provide("sketchSpaceDesigner.designer.modes.Zoom");

dojo.require("sketchSpaceDesigner.designer.modes.Mode");

dojo.declare("sketchSpaceDesigner.designer.modes.Zoom", [sketchSpaceDesigner.designer.modes.Mode], {
  zoomFactor: 0.15,
  mouseFactor: 0.3,
  mouseDown: undefined,
  enable: function () {
    this.inherited(arguments);
    var mode = this;

    this.onMouseWheelHandle = dojo.connect(this.designer.container, (!dojo.isMozilla ? "onmousewheel" : "DOMMouseScroll"), function(e){
      var scroll = e[(!dojo.isMozilla ? "wheelDelta" : "detail")] * (!dojo.isMozilla ? 1 : -1);
      mode.onMouseWheel(e, scroll);
    });
    this.onKeyUpHandle = dojo.connect(document, "onkeyup", this, function (event) { mode.onKeyUp(event); });
    this.onMouseMoveHandle = dojo.connect(this.designer.container, "onmousemove", this, this.onMouseMove);
    this.onMouseUpHandle = dojo.connect(this.designer.container, "onmouseup", this, this.onMouseUp);
    this.onMouseDownHandle = dojo.connect(this.designer.container, "onmousedown", this, this.onMouseDown);
  },
  disable: function () {
    dojo.disconnect(this.onMouseDownHandle);
    dojo.disconnect(this.onMouseUpHandle);
    dojo.disconnect(this.onMouseMoveHandle);
    dojo.disconnect(this.onKeyUpHandle);
    dojo.disconnect(this.onMouseWheelHandle);
    this.inherited(arguments);
  },
  onMouseWheel: function (event, scroll) {
    scroll *= this.mouseFactor;
    if (scroll < 0)
      scroll = 1.0 / (1.0 - this.zoomFactor * scroll);
    else
      scroll = 1.0 + this.zoomFactor * scroll;
    this.onZoom(scroll, event.layerX, event.layerY);
  },
  onKeyUp: function (event) {
    if (event.keyCode == 38 && event.ctrlKey && !event.altKey && !event.shiftKey) {
      this.onZoom(1.0 + this.zoomFactor);
    } else if (event.keyCode == 40 && event.ctrlKey && !event.altKey && !event.shiftKey) {
      this.onZoom(1.0 / (1.0 + this.zoomFactor));
    }
  },
  onZoom: function (zoom, x, y) {
    if (x === undefined) x = this.designer.surface_size.width / 2;
    if (y === undefined) y = this.designer.surface_size.height / 2;

    var screenToCurrentZoomMatrix = dojox.gfx.matrix.invert(this.designer.surface_transform._getRealMatrix());

    var mouse = dojox.gfx.matrix.multiplyPoint(screenToCurrentZoomMatrix, x, y);
    this.designer.surface_transform.applyTransform(dojox.gfx.matrix.scaleAt(zoom, zoom, mouse.x, mouse.y));
  },
  onMouseMove: function(event) {
    if (this.mouseDown !== undefined && this.mouseDown.button == 1 && !this.mouseDown.ctrlKey && !this.mouseDown.altKey && !this.mouseDown.shiftKey) {
       var orig = this.getCurrentMouse(this.mouseDown, this.designer.surface);
       var mouse = this.getCurrentMouse(event, this.designer.surface);
       var move = dojox.gfx.matrix.translate(mouse.x - orig.x, mouse.y - orig.y);
       this.designer.surface_transform.setTransform(dojox.gfx.matrix.multiply(move, this.panOriginalMatrix));
    }
  },
  onMouseUp: function(event) {
    this.mouseDown = undefined;
  },
  onMouseDown: function(event) {
    this.mouseDown = event;
    this.panOrig = this.getCurrentMouse(this.mouseDown, this.designer.surface);
    this.panOriginalMatrix = this.designer.surface_transform.getTransform();
  },
});
