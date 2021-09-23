import {Component, HostListener, OnInit, ViewChild, ViewContainerRef} from '@angular/core';
import AppValues from "./common/app.values";
import {AppSearchService} from "./search/search.service";
import {MatSidenav} from "@angular/material/sidenav";
import {SideNavService} from "./services/sidenav.service";

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.sass']
})
export class AppComponent implements OnInit {
  @ViewChild('sidenav', {static: true}) public sidenav?: MatSidenav;
  public title: string = AppValues.appTitle;
  public appLogo: string = `/${AppValues.appLogo}`;
  public searchValue: string = '';
  public showFiller: boolean = true;

  @HostListener('window:resize', ['$event'])
  private checkWindowSize(): void {
    if (window.innerWidth < window.innerHeight) {
      this.sidenavService.close();
    } else {
      this.sidenavService.open();
    }
  }

  constructor(
      private appSearchService: AppSearchService,
      private sidenavService: SideNavService) {
  }

  public ngOnInit(): void {
    console.log("SSS" + this.sidenav);
    if(this.sidenav) {
     this.sidenavService.setSidenav(this.sidenav);
     this.checkWindowSize();
    }
  }

  public onToggleSidenav(): void {
    this.sidenavService.toggle();
  }

  public onSearch(value: string): void {
    this.appSearchService.searchChanges(value);
  }
  public onOpenUserInfo(): void {
    console.info('click onOpenUserInfo');
  }
  public onAddNew(): void {
    console.info('click onAddNew');
  }
}
