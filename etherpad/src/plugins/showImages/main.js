import("etherpad.log");
function init() {
 this.hooks = ['aceGetFilterStack'];
 
 this.description = 'Render images inline';
 this.aceGetFilterStack = aceGetFilterStack;
 this.install = install;
 this.uninstall = uninstall;
}

function install() {
 log.info("Installing inlineImages");
}

function uninstall() {
 log.info("Uninstalling inlineImages");
}

function aceGetFilterStack (options){
	var linestylefilter = options.linestylefilter;
	return  [  
				linestylefilter.getRegexpFilter( new RegExp("*.png", "g"), 'IAMGOR')
		    ];
}

