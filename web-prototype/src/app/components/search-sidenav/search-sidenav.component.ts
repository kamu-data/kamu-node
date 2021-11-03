import {Component, EventEmitter, Input, Output} from "@angular/core";

@Component({
  selector: 'app-search-sidenav',
  templateUrl: './search-sidenav.component.html'
})
export class SearchSidenavComponent {
    @Input() public searchValue: string;
    @Input() public isMobileView: boolean;
    @Output() public onInputSearch: EventEmitter<string> = new EventEmitter();


    public onSearch(value: string): void {
        this.onInputSearch.emit(value);
    }
}