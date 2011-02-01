import("etherpad.log");
import("faststatic");
import("etherpad.utils.*");
import("etherpad.globals.*");
import("etherpad.helpers");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");
import("sqlbase.sqlobj");
import("etherpad.pad.model");
import("etherpad.pro.pro_accounts");
import("fastJSON");


var readonlyPad_data = [];

function docBarDropdownsPad(arg) {
    helpers.includeCss('plugins/readonlyPad/readonlyPad.css');
    helpers.includeJs('plugins/readonlyPad/readonlyPad.js');
    
    return arg.template.include('readonlyPadDropdown.ejs', undefined, ['readonlyPad']);
}

function collabServerUserChanges(args) {
    var readonlyPad_return = true;
    model.accessPadGlobal(args.pad, function(pad) {
	var opts = pad.getPadOptionsObj();
	
	if(opts.view)
	{
	    var isReadOnly = opts.view.readonlyPadPolicy;
	    if(isReadOnly!=null)
	    {
		if(isReadOnly==true)
		{
		    if(!pro_accounts.isAccountSignedIn())
		    {
			readonlyPad_return = false;
		    }
		}
	    }
	}
    });
    return [readonlyPad_return];
}
