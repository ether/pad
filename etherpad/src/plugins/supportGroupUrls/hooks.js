import("etherpad.log");
import("sqlbase.sqlbase");
import("plugins.supportGroupUrls.controllers.groupUrls");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");


function handlePath() {
  return [
  	['/', groupUrls.render_main],
    [/^\/([^\/]+)$/, groupUrls.redirect_to_specs_path], // supercede  [/^\/([^\/]+)$/, pad_control.render_pad],
  	[/^\/specs\/$/, forward(groupUrls)],
	[/^\/specs\/([^\/]+\/)*$/, forward(groupUrls)],
	[/^\/specs\/([^\/]+\/)*/, groupUrls.render_page]
  
  ];
}