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
    @Output() public onOpenMenu: EventEmitter<null> = new EventEmitter();

    @ViewChild('appHeaderMenuButton') appHeaderMenuButton: ElementRef<HTMLElement>;


    public onSearch(value: string): void {
        this.onInputSearch.emit(value);
        setTimeout(() => {
          this.triggerMenuClick();
        }, 200)
    }
    public onAddNew(): void {
        this.addNew.emit();
    }
    public onOpenUserInfo(): void {
        this.userInfo.emit();
    }
    public onToggleMenu(): void {
        this.onOpenMenu.emit();
    }

    public triggerMenuClick(): void {
        const el: HTMLElement = this.appHeaderMenuButton.nativeElement;
        el.focus();
        el.click();
        el.blur();
    }
}