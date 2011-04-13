dojo.provide("sketchSpaceDesigner.designer.modes.ZoomPlus");

dojo.require("sketchSpaceDesigner.designer.modes.Zoom");
dojo.require("sketchSpaceDesigner.designer.bbox");

dojo.declare("sketchSpaceDesigner.designer.modes.ZoomPlus", [sketchSpaceDesigner.designer.modes.Zoom], {
  enable: function () {
    this.inherited(arguments);
    // Set some defaults
    this.designer.setOptions({zoomIn: true}, true);
    this.zoomInOption = new sketchSpaceDesigner.designer.widgets.OptionCheckBox({title:"Zoom in [SHIFT]:", optionsPath:"zoomIn", designer:this.designer});
    this.designer.ui.options.addChild(this.zoomInOption);
    this.designer.ui.options.layout();

  },
  disable: function () {
    this.inherited(arguments);
    this.zoomInOption.destroyRecursive();
    this.designer.ui.options.layout();
  },

  enableOutline: function() {
    if (this.outline !== undefined) return;
    if (this.orig !== undefined && this.mouse !== undefined) {
      var bbox = sketchSpaceDesigner.designer.bbox.Bbox().addPoints([this.orig, this.mouse]);

      this.outline = this.designer.surface.createGroup();

      this.outline.setTransform(dojox.gfx.matrix.translate(bbox.x, bbox.y));
      this.outline.originalMatrix = this.outline.matrix;

      this.outline.outlineOuter = dojox.gfx.utils.deserialize(this.outline, {shape:{type:"rect", x:-2, y:-2, width:bbox.width+4, height:bbox.height+4}, stroke:{color:{r:196,g:196,b:196,a:1},width:1, style:"solid"}});
      this.outline.outlineInner = dojox.gfx.utils.deserialize(this.outline, {shape:{type:"rect", x:0, y:0, width:bbox.width, height:bbox.height}, stroke:{color:{r:128,g:128,b:128,a:1},width:1, style:"solid"}});
    }
  },

  disableOutline: function () {
    if (this.outline !== undefined) {
      this.outline.removeShape();
      this.outline = undefined;
    }
  },

  updateOutline: function () {
    this.disableOutline();
    this.enableOutline();
  },

  onKeyDown: function (event) {
    this.inherited(arguments);
    if (event.keyCode == 16) { /* key=SHIFT */
      this.designer.setOptions({zoomIn: !this.designer.options.zoomIn});
    }
  },

  onKeyUp: function (event) {
    this.inherited(arguments);
    if (event.keyCode == 16) { /* key=SHIFT */
      this.designer.setOptions({zoomIn: !this.designer.options.zoomIn});
    }
  },

  onMouseDown: function (event) {
    this.inherited(arguments);
    if (event.button == 0) {
      this.orig = this.mouse = {x:event.layerX, y:event.layerY}
      this.enableOutline();
    }
  },

  onMouseUp: function (event) {
    this.inherited(arguments);
    if (event.button == 0) {
      this.disableOutline();
      if (this.orig == this.mouse) {
        if (this.designer.options.zoomIn)
 	  this.onZoom(1.0 + this.zoomFactor, this.orig.x, this.orig.y);
	else
	  this.onZoom(1.0 / (1.0 + this.zoomFactor), this.orig.x, this.orig.y);
      } else {
        var bbox = sketchSpaceDesigner.designer.bbox.Bbox().addPoints([this.orig, this.mouse]);
	var originalMatrix = this.designer.surface_transform._getRealMatrix();

        var hScale = this.designer.surface_size.width / bbox.width;
        var vScale = this.designer.surface_size.height / bbox.height;

        this.designer.surface_transform.setTransform(dojox.gfx.matrix.multiply(dojox.gfx.matrix.scale(Math.min(hScale, vScale)), dojox.gfx.matrix.translate(-bbox.x, -bbox.y), originalMatrix));
      }
    }
  },

  onMouseMove: function (event) {
    this.inherited(arguments);
    if (this.outline !== undefined) {
      this.mouse = {x:event.layerX, y:event.layerY};
      this.updateOutline();
    }
  },
});
