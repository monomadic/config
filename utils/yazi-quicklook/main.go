package main

/*
#cgo darwin CFLAGS: -x objective-c -fobjc-arc
#cgo darwin LDFLAGS: -framework Cocoa -framework QuickLookUI

#import <Cocoa/Cocoa.h>
#import <QuickLookUI/QuickLookUI.h>

@interface YZQLPreviewItem : NSObject <QLPreviewItem>
@property(nonatomic, strong) NSURL *previewItemURL;
@property(nonatomic, copy) NSString *previewItemTitle;
@end

@implementation YZQLPreviewItem
@end

@interface YZQLAppDelegate : NSObject <NSApplicationDelegate, QLPreviewPanelDataSource, QLPreviewPanelDelegate>
@property(nonatomic, strong) YZQLPreviewItem *item;
@end

@implementation YZQLAppDelegate

- (void)applicationDidFinishLaunching:(NSNotification *)notification {
	NSArray<NSString *> *args = [[NSProcessInfo processInfo] arguments];
	if (args.count < 2) {
		[NSApp terminate:nil];
		return;
	}

	NSString *path = args[1];
	if (![[NSFileManager defaultManager] fileExistsAtPath:path]) {
		[NSApp terminate:nil];
		return;
	}

	NSURL *url = [NSURL fileURLWithPath:path].standardizedURL;
	self.item = [YZQLPreviewItem new];
	self.item.previewItemURL = url;
	self.item.previewItemTitle = url.lastPathComponent;

	[NSApp setActivationPolicy:NSApplicationActivationPolicyRegular];
	[NSApp activateIgnoringOtherApps:YES];

	QLPreviewPanel *panel = [QLPreviewPanel sharedPreviewPanel];
	panel.dataSource = self;
	panel.delegate = self;
	[panel reloadData];
	[panel makeKeyAndOrderFront:nil];
}

- (BOOL)applicationShouldTerminateAfterLastWindowClosed:(NSApplication *)sender {
	return YES;
}

- (NSInteger)numberOfPreviewItemsInPreviewPanel:(QLPreviewPanel *)panel {
	return self.item == nil ? 0 : 1;
}

- (id<QLPreviewItem>)previewPanel:(QLPreviewPanel *)panel previewItemAtIndex:(NSInteger)index {
	return self.item;
}

@end

void RunYaziQuickLook(void) {
	@autoreleasepool {
		NSApplication *app = [NSApplication sharedApplication];
		YZQLAppDelegate *delegate = [YZQLAppDelegate new];
		app.delegate = delegate;
		[app run];
	}
}
*/
import "C"

func main() {
	C.RunYaziQuickLook()
}
