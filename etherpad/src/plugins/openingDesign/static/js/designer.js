dojo.require("dojox.gfx");
dojo.require("dojox.gfx.move");
dojo.require("dojox.gfx.utils");

function initGfx(){
  openingDesign.editorArea = {};
  openingDesign.editorArea.container = dojo.byId("openingDesignDebug");
  openingDesign.editorArea.surface = dojox.gfx.createSurface(openingDesign.editorArea.container, 300, 300);
  openingDesign.editorArea.surface_size = {width: 300, height: 300};

  dojo.connect(openingDesign.editorArea.container, "ondragstart",   dojo, "stopEvent");
  dojo.connect(openingDesign.editorArea.container, "onselectstart", dojo, "stopEvent");

  var editorArea = top.openingDesign.editorArea;
}

dojo.addOnLoad(initGfx);
