dojo.provide("sketchSpaceDesigner.designer");

dojo.require("dojox.gfx");
dojo.require("dojox.gfx.move");
dojo.require("dojox.gfx.utils");
dojo.require("dojox.gfx.matrix");
dojo.require("dojox.uuid.generateRandomUuid");

dojo.declare("sketchSpaceDesigner.designer.Designer", [], {
 constructor: function (container, width, height) {
    this.container = container;
    this.surface = dojox.gfx.createSurface(this.container, width, height);
    this.surface_size = {width: width, height: height};

    dojo.connect(this.container, "ondragstart",   dojo, "stopEvent");
    dojo.connect(this.container, "onselectstart", dojo, "stopEvent");
  }

});

dojo.addOnLoad(function (){
  sketchSpace.editorArea = new sketchSpaceDesigner.designer.Designer(dojo.byId("sketchSpaceDebug"), 300, 300);
});
