import {Component, HostListener, OnInit} from '@angular/core';
import AppValues from "./common/app.values";
import {AppSearchService} from "./search/search.service";
import {SideNavService} from "./services/sidenav.service";
import {Router} from "@angular/router";

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.sass']
})
export class AppComponent implements OnInit {
  public appLogo: string = `/${AppValues.appLogo}`;
  public isMobileView: boolean = false;
  public searchValue: string = '';

  @HostListener('window:resize', ['$event'])
  private checkWindowSize(): void {
    this.checkView();
  }

  constructor(
      private router: Router,
      private appSearchService: AppSearchService,
      private sidenavService: SideNavService
  ) { }

  public ngOnInit() {
    this.checkView();
  }

  private checkView(): void {
    this.isMobileView = AppValues.isMobileView();
  }

  public onInputSearch(searchValue: string) {
    this.router.navigate(['search']);
    this.appSearchService.searchChanges(searchValue);
    this.appSearchService.search(searchValue);
  }

  public onToggleSidenav() {
    this.sidenavService.toggle();
  }

  public onOpenUserInfo(): void {
    console.info('click onOpenUserInfo');
  }

  public onAddNew(): void {
    console.info('click onAddNew');
  }
}
