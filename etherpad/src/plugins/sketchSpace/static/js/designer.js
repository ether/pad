dojo.require("dojox.gfx");
dojo.require("dojox.gfx.move");
dojo.require("dojox.gfx.utils");
dojo.require("dojox.gfx.matrix");
dojo.require("dojox.uuid.generateRandomUuid");

function initGfx(){
  sketchSpace.editorArea = {};
  sketchSpace.editorArea.container = dojo.byId("sketchSpaceDebug");
  sketchSpace.editorArea.surface = dojox.gfx.createSurface(sketchSpace.editorArea.container, 300, 300);
  sketchSpace.editorArea.surface_size = {width: 300, height: 300};

  dojo.connect(sketchSpace.editorArea.container, "ondragstart",   dojo, "stopEvent");
  dojo.connect(sketchSpace.editorArea.container, "onselectstart", dojo, "stopEvent");

  var editorArea = top.sketchSpace.editorArea;
}

dojo.addOnLoad(initGfx);
