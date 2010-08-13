import("etherpad.log");
import("sqlbase.sqlbase");
import("plugins.padHierarchy.controllers.hierarchyController");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");


function handlePath() {
  return [
  	['/', hierarchyController.render_main],
    [/^\/([^\/]+)$/, hierarchyController.redirect_to_specs_path], // supercede  [/^\/([^\/]+)$/, pad_control.render_pad],
  	[/^\/specs\/$/, forward(hierarchyController)],
	[/^\/specs\/([^\/]+\/)*$/, forward(hierarchyController)],
	[/^\/specs\/([^\/]+\/)*/, hierarchyController.render_page]
  
  ];
}