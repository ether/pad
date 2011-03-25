dojo.provide("sketchSpaceDesigner.designer.modes.Zoom");

dojo.require("sketchSpaceDesigner.designer.modes.Mode");

dojo.declare("sketchSpaceDesigner.designer.modes.Zoom", [sketchSpaceDesigner.designer.modes.Mode], {
  zoomFactor: 0.05,
  enable: function () {
    this.inherited(arguments);
    var mode = this;

    this.onMouseWheelHandle = dojo.connect(this.designer.container, (!dojo.isMozilla ? "onmousewheel" : "DOMMouseScroll"), function(e){
      var scroll = e[(!dojo.isMozilla ? "wheelDelta" : "detail")] * (!dojo.isMozilla ? 1 : -1);
      mode.onZoom(e, scroll);
    });
  },
  disable: function () {
    this.inherited(arguments);
    dojo.disconnect(this.onMouseWheelHandle);
  },
  onZoom: function (event, scroll) {
    var screenToCurrentZoomMatrix = dojox.gfx.matrix.invert(this.designer.surface_transform._getRealMatrix());

    var mouse = dojox.gfx.matrix.multiplyPoint(screenToCurrentZoomMatrix, event.layerX, event.layerY);

   if (scroll < 0)
     scroll = 1.0/ (1.0 - this.zoomFactor * scroll);
   else
     scroll = 1.0 + this.zoomFactor * scroll;
   this.designer.surface_transform.applyTransform(dojox.gfx.matrix.scaleAt(scroll, scroll, mouse.x, mouse.y));
  }
});