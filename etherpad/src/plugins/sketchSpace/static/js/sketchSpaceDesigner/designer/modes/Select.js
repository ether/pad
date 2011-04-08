dojo.provide("sketchSpaceDesigner.designer.modes.Select");

dojo.require("sketchSpaceDesigner.designer.modes.Zoom");
dojo.require("dijit.form.Button");

dojo.declare("sketchSpaceDesigner.designer.modes.Select", [sketchSpaceDesigner.designer.modes.Zoom], {
  isOutlineMouseDown: false,
  isMoving: false,
  enable: function () {
    var designer = this;
    this.inherited(arguments);
    this.enableOutline();
    this.selectionUpdatedHandle = dojo.connect(this.designer.selection, "selectionUpdated", this, this.updateOutline);
    this.viewUpdatedHandle = dojo.connect(this.designer, "viewUpdated", this, this.updateOutline);

    this.applyOption = new dijit.layout._LayoutWidget({title:"Apply options [ENTER]:"});
    this.applyOption.addChild(new dijit.form.Button({label:"Apply", onClick: function () { designer.applyOptionsToSelection(); }}));
    this.designer.ui.options.addChild(this.applyOption);
    this.deleteOption = new dijit.layout._LayoutWidget({title:"Delete [DELETE]:"});
    this.deleteOption.addChild(new dijit.form.Button({label:"Delete", onClick: function () { designer.deleteSelection(); }}));
    this.designer.ui.options.addChild(this.deleteOption);
    this.designer.ui.options.layout();
  },

  disable: function () {
    dojo.disconnect(this.viewUpdatedHandle);
    dojo.disconnect(this.selectionUpdatedHandle);
    this.disableOutline();
    this.inherited(arguments);
    this.applyOption.destroyRecursive();
    this.deleteOption.destroyRecursive();
    this.designer.ui.options.layout();
  },

  enableOutline: function() {
    var bbox = this.designer.selection.getBbox();

    if (bbox.x !== undefined) {
      this.outline = this.designer.surface.createGroup();

      this.outline.setTransform(dojox.gfx.matrix.translate(bbox.x, bbox.y));
      this.outline.originalMatrix = this.outline.matrix;

      this.outline.outlineRect = dojox.gfx.utils.deserialize(this.outline, {shape:{type:"rect", x:0, y:0, width:bbox.width, height:bbox.height}, stroke:{color:{r:196,g:196,b:196,a:1},width:1, style:"solid"}});

      this.outline.outlineCornerTL = dojox.gfx.utils.deserialize(this.outline, {shape:{type:"rect", x:-2, y:-2, width:4, height:4}, stroke:{color:{r:128,g:128,b:128,a:1},width:1}, fill:{r:196,g:196,b:196,a:1}});
      this.outline.outlineCornerBL = dojox.gfx.utils.deserialize(this.outline, {shape:{type:"rect", x:-2, y:bbox.height-2, width:4, height:4}, stroke:{color:{r:128,g:128,b:128,a:1},width:1}, fill:{r:196,g:196,b:196,a:1}});
      this.outline.outlineCornerTH = dojox.gfx.utils.deserialize(this.outline, {shape:{type:"rect", x:bbox.width-2, y:-2, width:4, height:4}, stroke:{color:{r:128,g:128,b:128,a:1},width:1}, fill:{r:196,g:196,b:196,a:1}});
      this.outline.outlineCornerBH = dojox.gfx.utils.deserialize(this.outline, {shape:{type:"rect", x:bbox.width-2, y:bbox.height-2, width:4, height:4}, stroke:{color:{r:128,g:128,b:128,a:1},width:1}, fill:{r:196,g:196,b:196,a:1}});

      this.outline.onMouseDownHandle = this.outline.connect("onmousedown", this, this.onOutlineMouseDown);
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

  applyOptionsToSelection: function () {
    this.designer.selection.applyToShapes(function (designer) {
      designer.setShapeFillAndStroke(this, designer.options);
    }, this.designer);
    this.designer.selection.applyToShapes("save");
  },

  deleteSelection: function () {
    this.designer.selection.applyToShapes("removeShape");
  },

  onShapeMouseDown: function (shape, event) {
    this.inherited(arguments);
    this.onOutlineMouseDown(event);
  },

  onShapeMouseUp: function (shape, event) {
    this.inherited(arguments);
    if (!this.isOutlineMoving)
      this.designer.selection.toggleShape(shape, !event.ctrlKey);
  },

  onKeyUp: function (event) {
    this.inherited(arguments);
    if (event.keyCode == 13) {
      this.applyOptionsToSelection();
    } else if (event.keyCode == 46) {
      this.deleteSelection();
    } else if (event.keyCode == 36) {
      this.designer.selection.applyToShapes(
	function(){
	  this.zOrderMoved = true; this.moveToFront();
//	  console.log('Move ' + this.objId + ' to front');
	}
      );
      this.designer.selection.applyToShapes("save");
    } else if (event.keyCode == 35) {
      this.designer.selection.applyToShapes(
	function(){
	  this.zOrderMoved = true; this.moveToBack();
//	  console.log('Move ' + this.objId + ' to back');
	}
      );
      this.designer.selection.applyToShapes("save");
    } else if (event.keyCode == 34) {
      var move = {};
      this.designer.selection.applyToShapes(
	function(){
	  this.zOrderMoved = true;
	  move[this.objId]=true;
	}
      );

      var shapes={};
      this.designer.forEachObjectShape(function(shape){shapes[shape.objId]=shape;});
      var img = this.designer.images[this.designer.currentImage];
      var myOrder = img.order.slice(0);
      for(var i=0; i<myOrder.length; i++){
	if(myOrder[i] in move){
	  if(i == 0) {
	    continue;

	  }
	  var tmp=myOrder[i-1];
	  if(myOrder[i-1] in move) {
	    continue;
	  }

	  myOrder[i-1] = myOrder[i];
	  myOrder[i]=tmp;
	}
      }
      for(var i=0; i<myOrder.length; i++){
	if(shapes[myOrder[i]]){
	  shapes[myOrder[i]].moveToFront();
	}
      }

      this.designer.selection.applyToShapes("save");
    } else if (event.keyCode == 33) {
      var move = {};
      this.designer.selection.applyToShapes(
	function(){
	  this.zOrderMoved = true;
	  move[this.objId]=true;
	}
      );

      var shapes={};
      this.designer.forEachObjectShape(function(shape){shapes[shape.objId]=shape;});
      var img = this.designer.images[this.designer.currentImage];
      var myOrder = img.order.slice(0);
      for(var i=myOrder.length-1; i>=0; i--){
	if(myOrder[i] in move){
	  if(i == (myOrder-length-1)) {
	    continue;
	  }
	  var tmp=myOrder[i+1];
	  if(myOrder[i+1] in move) {
	    continue;
	  }

	  myOrder[i+1] = myOrder[i];
	  myOrder[i]=tmp;
	}
      }
      for(var i=0; i<myOrder.length; i++){
	if(shapes[myOrder[i]]){
	  shapes[myOrder[i]].moveToFront();
	}
      }

      this.designer.selection.applyToShapes("save");
    }
  },

  onMouseUp: function(event) {
    this.inherited(arguments);
    if (this.isOutlineMoving) {
      this.designer.selection.applyToShapes("save");
      this.isOutlineMoving = false;
    }
    this.isOutlineMouseDown = false;
  },

  onMouseMove: function(event) {
    this.inherited(arguments);

    if (!this.isOutlineMouseDown || !this.outline) return;

    this.isOutlineMoving = true;

    var move = this.getCurrentMove(event);
    this.outline.setTransform(dojox.gfx.matrix.multiply(this.outline.originalMatrix, move));

    move = this.getCurrentMove(event, this.designer.selection.parent, this.designer.selection.orig);
    this.designer.selection.applyToShapes(function () {
      this.setTransform(dojox.gfx.matrix.multiply(this.originalMatrix, move));
    });
  },

  getContainerShape: function () { return this.designer.surface; },

  onOutlineMouseDown: function(event) {
    this.isOutlineMouseDown = true;
    this.orig = this.getCurrentMouse(event);
    this.designer.selection.orig = this.getCurrentMouse(event, this.designer.selection.parent);
    if (!this.outline) return;
    this.outline.originalMatrix = this.outline.matrix;
    this.designer.selection.applyToShapes(function () {
      this.originalMatrix = this.matrix;
    });
  },

});
