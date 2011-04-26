dojo.provide("sketchSpaceDesigner.designer.modes.AddPathPolyline");

dojo.require("sketchSpaceDesigner.designer.modes.EditPath");
dojo.require("sketchSpaceDesigner.utils");

dojo.declare("sketchSpaceDesigner.designer.modes.AddPathPolyline.Path", [sketchSpaceDesigner.designer.modes.EditPath.prototype.Path], {
  setOptions: function (options) {
   this.inherited(arguments,
		  [sketchSpaceDesigner.utils.setObject({
		     isClosed: false,
		     isLine: true,
		   }, options, true)]);
  },
});

dojo.declare("sketchSpaceDesigner.designer.modes.AddPathPolyline", [sketchSpaceDesigner.designer.modes.EditPath], {
  Path: sketchSpaceDesigner.designer.modes.AddPathPolyline.Path,
  enable: function () {
    this.inherited(arguments);
    // Set some defaults
    this.designer.setOptions({isStraight: false}, true);
    this.isStraightOption = new sketchSpaceDesigner.designer.widgets.OptionCheckBox({title:"Straighten [SHIFT]:", optionsPath:"isStraight", designer:this.designer});
    this.designer.ui.options.addChild(this.isStraightOption);
    this.designer.ui.options.layout();

  },
  disable: function () {
    this.inherited(arguments);
    if (this.path !== undefined) {
      this.path.shape.removeShape();
    }
    this.isStraightOption.destroyRecursive();
    this.designer.ui.options.layout();
  },
  onKeyDown: function (event) {
    this.inherited(arguments);
    if (event.keyCode == dojo.keys.SHIFT) {
      this.designer.setOptions({isStraight: !this.designer.options.isStraight});
    }
  },
  onKeyUp: function (event) {
    this.inherited(arguments);
    if (event.keyCode == dojo.keys.SHIFT) {
      this.designer.setOptions({isStraight: !this.designer.options.isStraight});
    }
  },
  onMouseDown: function (event) {
    this.inherited(arguments);
    if (event.button == dojo.mouseButtons.LEFT) {
      if (this.path !== undefined)
        this.beginSection(this.getCurrentMouse(event));
      else
        this.begin(this.getCurrentMouse(event));
    }
  },
  onMouseUp: function (event) {
    this.inherited(arguments);
    if (   event.button == dojo.mouseButtons.RIGHT
	|| (   this.lastMouseUpEvent
	    && this.lastMouseUpEvent.layerX == event.layerX
	    && this.lastMouseUpEvent.layerY == event.layerY
	    && this.lastMouseUpEvent.button == dojo.mouseButtons.LEFT
            && event.button == dojo.mouseButtons.LEFT))  {
      if (this.path !== undefined) {
        this.done()
      }
    }
    this.lastMouseUpEvent = event;
  },
  onMouseMove: function (event) {
    this.inherited(arguments);
    this.addPoint(this.getCurrentMouse(event));
  },
});
