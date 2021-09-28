import {Component, EventEmitter, Input, OnInit, Output} from "@angular/core";

@Component({
  selector: 'app-search-header',
  templateUrl: './search-header.component.html'
})
export class SearchHeaderComponent {
    @Input() public searchValue: string;
    @Input() public appLogo: string;
    @Input() public isMobileView: boolean;
    @Output() public onInputSearch: EventEmitter<string> = new EventEmitter();
    @Output() public addNew: EventEmitter<null> = new EventEmitter();
    @Output() public userInfo: EventEmitter<null> = new EventEmitter();


    public onSearch(value: string): void {
        this.onInputSearch.emit(value);
    }
    public onAddNew(): void {
        this.addNew.emit();
    }
    public onOpenUserInfo(): void {
        this.userInfo.emit();
    }
}