dojo.provide("sketchSpaceDesigner.designer.modes.Zoom");

dojo.require("sketchSpaceDesigner.designer.modes.Mode");

dojo.declare("sketchSpaceDesigner.designer.modes.Zoom", [sketchSpaceDesigner.designer.modes.Mode], {
  zoomFactor: 0.15,
  mouseFactor: 0.3,
  enable: function () {
    this.inherited(arguments);
    var mode = this;

    this.onMouseWheelHandle = dojo.connect(this.designer.container, (!dojo.isMozilla ? "onmousewheel" : "DOMMouseScroll"), function(e){
      var scroll = e[(!dojo.isMozilla ? "wheelDelta" : "detail")] * (!dojo.isMozilla ? 1 : -1);
      mode.onMouseWheel(e, scroll);
    });
    this.onKeyUpHandle = dojo.connect(document, "onkeyup", this, function (event) { mode.onKeyUp(event); });
  },
  disable: function () {
    this.inherited(arguments);
    dojo.disconnect(this.onKeyUpHandle);
    dojo.disconnect(this.onMouseWheelHandle);
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
  }
});