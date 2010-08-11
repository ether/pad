import("etherpad.log");
import("sqlbase.sqlbase");
import("plugins.supportGroupUrls.controllers.groupUrls");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");
import("etherpad.control.pad.pad_control");

function handlePath() {
  return [
  	[/^\/specs\/$/, forward(groupUrls)],
	[/^\/specs\/([^\/]+\/)+$/, forward(groupUrls)],
	[/^\/specs\/([^\/]+\/)+/, groupUrls.render_page]
  
  ];
}