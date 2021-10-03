import {Component, ElementRef, EventEmitter, Input, OnInit, Output, ViewChild} from "@angular/core";

@Component({
  selector: 'app-header',
  templateUrl: './app-header.component.html'
})
export class AppHeaderComponent {
    @Input() public searchValue: string;
    @Input() public appLogo: string;
    @Input() public isMobileView: boolean;

    @Output() public onInputSearch: EventEmitter<string> = new EventEmitter();
    @Output() public addNew: EventEmitter<null> = new EventEmitter();
    @Output() public userInfo: EventEmitter<null> = new EventEmitter();

    @ViewChild('appHeaderMenuButton') appHeaderMenuButton: ElementRef<HTMLElement>;


    public onSearch(event: InputEvent, value: string): void {
        debugger

        this.onInputSearch.emit(value);
        setTimeout(() => {
            if(this.isMobileView) {
                this.triggerMenuClick();
            }

            (event.target as HTMLElement).blur();
        }, 200)
    }
    public onAddNew(): void {
        this.addNew.emit();
    }
    public onOpenUserInfo(): void {
        this.userInfo.emit();
    }

    public triggerMenuClick(): void {
        const el: HTMLElement = this.appHeaderMenuButton.nativeElement;
        el.focus();
        el.click();
        el.blur();
    }
}