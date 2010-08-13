HierarchyTest = TestCase("HierarchyTest");

HierarchyTest.prototype.testNestingDetection = function(){
	var titles = [
		"maverick-software-center-lobby",
		"maverick maverick-installer",
		"maverick-installer-partitioner",
		"maverick-installer-partitioner-when-other-os-installed",
		"maverick-installer-setup-process",
		"maverick-installer-setup-process-get-ready",
		"maverick-installer-setup-process-keyboard-layout",
		"maverick-installer-setup-process-slideshow",
		"maverick-installer-setup-process-welcome",
		"maverick-installer-setup-process-wifi-prompt",
		"maverick-installer-user-account-create",
		"maverick-software-center",
		"maverick-software-center-apturl",
		"maverick-software-center-buy",
		"maverick-software-center-buying",
		"maverick-software-center-deauthorizing",
		"maverick-software-center-department",
		"maverick-software-center-department-subsection",
		"maverick-software-center-lobby",
		"maverick-software-center-reinstalling-previous-purchases",
		"maverick-software-center-whats-new",
		"people-michaelforrest-ubuntu-spec-editor",
		"people-michaelforrest-ubuntu-spec-editor-authentication"
	];
	
	var expectation = [
		{
			"maverick": [
			{
				"software-center": [
					"lobby", "apturl", "buy", "buying", "deauthorizing", {
						"department": ["subsection"]
					}, "reinstalling-previous-purchases", "whats-new"]
			}, {
				"installer": [{
					"partitioner": ["when-other-os-installed"]
				}, {
					"setup-process": ["get-ready", "keyboard-layout", "slideshow", "welcome", "wifi-prompt"]
				}, "user-account-create"]
			}]
		},
		{
			"people": [{
				"michaelforrest": {
					"ubuntu-spec-editor": ["authentication"]
				}
			}]
		}
	];
	assertEquals(expectation,getHierarchy(titles), "should have right number of top-level structures");
	
};

  
  